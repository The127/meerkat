mod auth;
mod subdomain;

use std::sync::Arc;

use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;

use meerkat_application::context::RequestContext;
use meerkat_application::ports::error_observer::ErrorReport;

use crate::state::AppState;

pub(crate) use auth::authenticate;
pub(crate) use subdomain::resolve_subdomain;

pub(crate) async fn request_context(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    let ctx = Arc::new(RequestContext::new(state.context.clone()));
    request.extensions_mut().insert(ctx);
    next.run(request).await
}

pub(crate) async fn error_observer(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let response = next.run(request).await;

    if let Some(report) = response.extensions().get::<ErrorReport>() {
        state.context.error_observer.observe(report).await;
    } else if response.status().is_client_error() || response.status().is_server_error() {
        tracing::warn!(
            method = %method,
            uri = %uri,
            status = response.status().as_u16(),
            "unhandled error response",
        );
    }

    response
}
