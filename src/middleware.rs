use std::{path::Path, sync::Arc};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use minijinja::context;

use crate::{AppState, assets};

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

pub async fn maintenance(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    if state.maintenance {
        let scripts = assets::resolve_scripts(
            Path::new("client/main.ts"),
            #[cfg(not(feature = "debug"))]
            Some(&state.manifest),
            #[cfg(feature = "debug")]
            None,
        );

        let css = assets::resolve_css(
            Path::new("client/main.ts"),
            #[cfg(not(feature = "debug"))]
            Some(&state.manifest),
            #[cfg(feature = "debug")]
            None,
        );
        let env = state.loader.acquire_env().unwrap();

        let res = env
            .get_template("service-unavailable.njk")
            .unwrap()
            .render(context! { css => css, scripts => scripts })
            .unwrap();

        let mut response = Html(res).into_response();

        *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;

        return response;
    }

    next.run(request).await
}
