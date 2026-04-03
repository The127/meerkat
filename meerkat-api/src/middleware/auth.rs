use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jsonwebtoken::{decode, decode_header, jwk::Jwk, DecodingKey, TokenData, Validation};

use std::collections::HashSet;

use meerkat_application::auth_context::AuthContext;
use meerkat_application::ports::oidc_config_read_store::OidcConfigReadModel;
use meerkat_domain::models::member::Sub;
use meerkat_domain::models::permission::EffectivePermission;

use crate::error::ErrorDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

#[derive(serde::Deserialize)]
struct Claims {
    iss: Option<String>,
    aud: Option<Aud>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Aud {
    Single(String),
    Multiple(Vec<String>),
}

impl Aud {
    fn contains(&self, value: &str) -> bool {
        match self {
            Aud::Single(s) => s == value,
            Aud::Multiple(v) => v.iter().any(|s| s == value),
        }
    }
}

pub(crate) async fn authenticate(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    match authenticate_inner(&state, request, next).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn authenticate_inner(
    state: &AppState,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, Response> {
    let token = extract_bearer_token(&request)?;
    let header = decode_header(token).map_err(|_| unauthorized("invalid token"))?;
    let unverified_claims = decode_claims_unverified(token)?;

    let resolved_org = request
        .extensions()
        .get::<ResolvedOrganization>()
        .cloned()
        .ok_or_else(internal_error)?;

    let config = state
        .oidc_config_read_store
        .find_active_by_org_id(&resolved_org.id)
        .await
        .map_err(|_| internal_error())?;

    validate_issuer_and_audience(&unverified_claims, &config)?;

    let jwk = resolve_decoding_jwk(state, &config, header.kid.as_deref()).await?;

    let decoding_key = DecodingKey::from_jwk(&jwk)
        .map_err(|_| unauthorized("unsupported key type"))?;

    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[config.audience.as_str()]);
    validation.set_issuer(&[config.issuer_url.as_str()]);

    let token_data: TokenData<serde_json::Value> =
        decode(token, &decoding_key, &validation).map_err(|_| unauthorized("token validation failed"))?;

    let claims = &token_data.claims;
    let claim_mapping = &config.claim_mapping;

    let sub_value = claims
        .get(claim_mapping.sub_claim().as_str())
        .and_then(|v| v.as_str())
        .ok_or_else(|| unauthorized("missing sub claim"))?;

    let sub = Sub::new(sub_value).map_err(|_| unauthorized("invalid sub claim"))?;

    let role_values = extract_string_values(claims, claim_mapping.role_claim().as_str());
    let role_refs: Vec<&str> = role_values.iter().map(|s| s.as_str()).collect();
    let org_roles = claim_mapping.resolve_roles(&role_refs);

    if org_roles.is_empty() {
        return Err(unauthorized("no matching org role"));
    }

    let preferred_name = claims
        .get(claim_mapping.name_claim().as_str())
        .and_then(|v| v.as_str())
        .unwrap_or(sub_value);

    let member_id = state
        .member_repository
        .find_or_create(&resolved_org.id, &sub, preferred_name)
        .await
        .map_err(|_| internal_error())?;

    let permissions: HashSet<EffectivePermission> = org_roles
        .iter()
        .flat_map(|role| role.permissions())
        .map(EffectivePermission::Org)
        .collect();

    let auth_context = AuthContext {
        sub,
        org_id: resolved_org.id,
        org_roles,
        member_id,
        preferred_name: preferred_name.to_string(),
        permissions,
    };

    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

fn extract_string_values(claims: &serde_json::Value, key: &str) -> Vec<String> {
    match claims.get(key) {
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    }
}

#[allow(clippy::result_large_err)]
fn validate_issuer_and_audience(claims: &Claims, config: &OidcConfigReadModel) -> Result<(), Response> {
    match &claims.iss {
        Some(iss) if iss == config.issuer_url.as_str() => {}
        Some(_) => return Err(unauthorized("issuer mismatch")),
        None => return Err(unauthorized("missing issuer claim")),
    }

    match &claims.aud {
        Some(aud) if aud.contains(config.audience.as_str()) => {}
        Some(_) => return Err(unauthorized("audience mismatch")),
        None => return Err(unauthorized("missing audience claim")),
    }

    Ok(())
}

async fn resolve_decoding_jwk(
    state: &AppState,
    config: &OidcConfigReadModel,
    kid: Option<&str>,
) -> Result<Jwk, Response> {
    let discovery_url = discovery_url_for_config(config);

    let jwks_url = state
        .oidc_discovery_provider
        .resolve_jwks_uri(&discovery_url)
        .await
        .map_err(|_| internal_error())?;

    let jwk_value = state
        .jwks_provider
        .resolve_jwk(&jwks_url, kid)
        .await
        .map_err(|_| unauthorized("signing key not found"))?;

    serde_json::from_value(jwk_value).map_err(|_| internal_error())
}

#[allow(clippy::result_large_err)]
fn extract_bearer_token(request: &axum::extract::Request) -> Result<&str, Response> {
    let header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| unauthorized("missing authorization header"))?;

    header
        .strip_prefix("Bearer ")
        .ok_or_else(|| unauthorized("invalid authorization scheme"))
}

#[allow(clippy::result_large_err)]
fn decode_claims_unverified(token: &str) -> Result<Claims, Response> {
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_aud = false;
    validation.validate_exp = false;

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&[]), &validation)
        .map_err(|_| unauthorized("malformed token"))?;

    Ok(token_data.claims)
}

fn discovery_url_for_config(config: &OidcConfigReadModel) -> String {
    config
        .discovery_url
        .as_ref()
        .map(|u| u.as_str().to_string())
        .unwrap_or_else(|| {
            let issuer = config.issuer_url.as_str().trim_end_matches('/');
            format!("{issuer}/.well-known/openid-configuration")
        })
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(ErrorDto {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

fn unauthorized(message: &str) -> Response {
    error_response(StatusCode::UNAUTHORIZED, "unauthorized", message)
}

fn internal_error() -> Response {
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        "an unexpected error occurred",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use meerkat_application::ports::oidc_config_read_store::OidcConfigReadModel;
    use meerkat_domain::models::oidc_config::{Audience, ClientId, OidcConfigId};
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::shared::url::Url;
    use meerkat_domain::testing::test_claim_mapping;

    fn test_config(discovery_url: Option<&str>) -> OidcConfigReadModel {
        OidcConfigReadModel {
            id: OidcConfigId::new(),
            organization_id: OrganizationId::new(),
            name: "Test".to_string(),
            client_id: ClientId::new("client-1").unwrap(),
            issuer_url: Url::new("https://auth.example.com").unwrap(),
            audience: Audience::new("my-api").unwrap(),
            discovery_url: discovery_url.map(|u| Url::new(u).unwrap()),
            claim_mapping: test_claim_mapping(),
        }
    }

    #[test]
    fn given_explicit_discovery_url_then_uses_it() {
        // arrange
        let config = test_config(Some("https://auth.example.com/oidc/.well-known/openid-configuration"));

        // act
        let url = discovery_url_for_config(&config);

        // assert
        assert_eq!(url, "https://auth.example.com/oidc/.well-known/openid-configuration");
    }

    #[test]
    fn given_no_discovery_url_then_derives_from_issuer() {
        // arrange
        let config = test_config(None);

        // act
        let url = discovery_url_for_config(&config);

        // assert
        assert_eq!(url, "https://auth.example.com/.well-known/openid-configuration");
    }

    #[test]
    fn given_issuer_with_trailing_slash_then_derives_without_double_slash() {
        // arrange
        let mut config = test_config(None);
        config.issuer_url = Url::new("https://auth.example.com/").unwrap();

        // act
        let url = discovery_url_for_config(&config);

        // assert
        assert_eq!(url, "https://auth.example.com/.well-known/openid-configuration");
    }

    #[test]
    fn given_valid_bearer_header_then_extracts_token() {
        // arrange
        let request = Request::builder()
            .header("authorization", "Bearer eyJhbGciOiJSUzI1NiJ9.test.sig")
            .body(axum::body::Body::empty())
            .unwrap();

        // act
        let result = extract_bearer_token(&request);

        // assert
        assert_eq!(result.unwrap(), "eyJhbGciOiJSUzI1NiJ9.test.sig");
    }

    #[test]
    fn given_missing_auth_header_then_returns_error() {
        // arrange
        let request = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();

        // act
        let result = extract_bearer_token(&request);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn given_basic_auth_scheme_then_returns_error() {
        // arrange
        let request = Request::builder()
            .header("authorization", "Basic dXNlcjpwYXNz")
            .body(axum::body::Body::empty())
            .unwrap();

        // act
        let result = extract_bearer_token(&request);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn given_single_aud_claim_then_contains_works() {
        // arrange
        let aud = Aud::Single("my-api".to_string());

        // act / assert
        assert!(aud.contains("my-api"));
        assert!(!aud.contains("other-api"));
    }

    #[test]
    fn given_multiple_aud_claims_then_contains_works() {
        // arrange
        let aud = Aud::Multiple(vec!["api-1".to_string(), "api-2".to_string()]);

        // act / assert
        assert!(aud.contains("api-1"));
        assert!(aud.contains("api-2"));
        assert!(!aud.contains("api-3"));
    }

    #[test]
    fn given_string_role_claim_then_extracts_single_value() {
        // arrange
        let claims = serde_json::json!({"roles": "admin"});

        // act
        let values = extract_string_values(&claims, "roles");

        // assert
        assert_eq!(values, vec!["admin"]);
    }

    #[test]
    fn given_array_role_claim_then_extracts_all_values() {
        // arrange
        let claims = serde_json::json!({"roles": ["admin", "member"]});

        // act
        let values = extract_string_values(&claims, "roles");

        // assert
        assert_eq!(values, vec!["admin", "member"]);
    }

    #[test]
    fn given_missing_role_claim_then_returns_empty() {
        // arrange
        let claims = serde_json::json!({"sub": "user-1"});

        // act
        let values = extract_string_values(&claims, "roles");

        // assert
        assert!(values.is_empty());
    }
}
