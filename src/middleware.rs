use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Redirect, Response},
};

use crate::AppState;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    request: Request,
    next: Next,
) -> axum::response::Result<Response> {
    let Some(sub) = session.get::<String>("user").await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to read from session",
        )
    })?
    else {
        return Err(Redirect::to("/").into());
    };

    let response = next.run(request).await;

    // do something with `response`...

    Ok(response)
}
