use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;

use meerkat_application::ports::error_observer::ErrorReport;

use crate::state::AppState;

pub(crate) async fn error_observer(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;

    if let Some(report) = response.extensions().get::<ErrorReport>() {
        state.context.error_observer.observe(report).await;
    }

    response
}
