use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;

use meerkat_domain::models::organization::OrganizationSlug;

use crate::error::ErrorDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

pub(crate) async fn resolve_subdomain(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let host_without_port = strip_port(host);

    let subdomain = extract_subdomain(host_without_port, &state.base_domain);

    let (slug_to_lookup, is_master) = match subdomain {
        Some(sub) => match OrganizationSlug::new(sub) {
            Ok(slug) => {
                let is_master = slug.as_str() == state.master_org_slug;
                (slug, is_master)
            }
            Err(_) => return not_found_response(),
        },
        None => {
            // Bare domain → master org
            match OrganizationSlug::new(&state.master_org_slug) {
                Ok(slug) => (slug, true),
                Err(_) => return internal_error_response(),
            }
        }
    };

    match state.org_read_store.find_by_slug(&slug_to_lookup).await {
        Ok(Some(org)) => {
            request.extensions_mut().insert(ResolvedOrganization {
                id: org.id,
                slug: org.slug,
                name: org.name,
                is_master,
            });
            next.run(request).await
        }
        Ok(None) => not_found_response(),
        Err(_) => internal_error_response(),
    }
}

/// Strip port from a Host header value.
/// Handles both regular hosts (`acme.dev:3030`) and bracketed IPv6 (`[::1]:3030`).
fn strip_port(host: &str) -> &str {
    if host.starts_with('[') {
        // IPv6: [::1]:port or [::1]
        host.find("]:")
            .map(|i| &host[..=i])
            .unwrap_or(host)
    } else {
        host.rsplit_once(':')
            .filter(|(_, port)| port.chars().all(|c| c.is_ascii_digit()))
            .map(|(h, _)| h)
            .unwrap_or(host)
    }
}

fn extract_subdomain<'a>(host: &'a str, base_domain: &str) -> Option<&'a str> {
    if host == base_domain {
        return None;
    }

    host.strip_suffix(base_domain)
        .and_then(|prefix| prefix.strip_suffix('.'))
        .filter(|s| !s.is_empty())
}

fn not_found_response() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorDto {
            code: "organization_not_found".to_string(),
            message: "organization not found".to_string(),
        }),
    )
        .into_response()
}

fn internal_error_response() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorDto {
            code: "internal_error".to_string(),
            message: "an unexpected error occurred".to_string(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_host_with_port_then_strips_port() {
        // arrange / act / assert
        assert_eq!(strip_port("acme.dev:3030"), "acme.dev");
    }

    #[test]
    fn given_host_without_port_then_returns_as_is() {
        // arrange / act / assert
        assert_eq!(strip_port("acme.dev"), "acme.dev");
    }

    #[test]
    fn given_ipv6_with_port_then_strips_port() {
        // arrange / act / assert
        assert_eq!(strip_port("[::1]:3030"), "[::1]");
    }

    #[test]
    fn given_ipv6_without_port_then_returns_as_is() {
        // arrange / act / assert
        assert_eq!(strip_port("[::1]"), "[::1]");
    }

    #[test]
    fn given_bare_domain_then_no_subdomain() {
        // arrange / act / assert
        assert_eq!(extract_subdomain("meerkat.dev", "meerkat.dev"), None);
    }

    #[test]
    fn given_subdomain_then_extracts_it() {
        // arrange / act
        let result = extract_subdomain("acme.meerkat.dev", "meerkat.dev");

        // assert
        assert_eq!(result, Some("acme"));
    }

    #[test]
    fn given_nested_subdomain_then_extracts_full_prefix() {
        // arrange / act
        let result = extract_subdomain("eu.acme.meerkat.dev", "meerkat.dev");

        // assert
        assert_eq!(result, Some("eu.acme"));
    }

    #[test]
    fn given_unrelated_host_then_no_subdomain() {
        // arrange / act / assert
        assert_eq!(extract_subdomain("evil.com", "meerkat.dev"), None);
    }

    #[test]
    fn given_empty_host_then_no_subdomain() {
        // arrange / act / assert
        assert_eq!(extract_subdomain("", "meerkat.dev"), None);
    }

    #[test]
    fn given_base_domain_as_suffix_without_dot_separator_then_no_subdomain() {
        // arrange / act — "fooreignmeerkat.dev" should NOT match
        let result = extract_subdomain("fooreignmeerkat.dev", "meerkat.dev");

        // assert
        assert_eq!(result, None);
    }
}
