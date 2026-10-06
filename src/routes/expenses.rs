use axum::{
    Router,
    extract::{Path as PathParam, State},
    response::{Html, Redirect},
    routing::get,
};
use minijinja::context;
use std::{path::Path, sync::Arc};
use tracing::debug;

use crate::{AppState, PAGES, assets, middleware};

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list))
        .route("/new", get(new_form))
        .route("/{id}", get(get_receipt))
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            middleware::auth,
        ))
}

async fn list(State(state): State<Arc<AppState>>) -> axum::response::Result<Html<String>> {
    let scripts = assets::resolve_scripts(
        Path::new("client/main.ts"),
        #[cfg(not(feature = "debug"))]
        Some(&state.manifest),
        #[cfg(feature = "debug")]
        None,
    );
    debug!("Loaded scripts");

    let css = assets::resolve_css(
        Path::new("client/main.ts"),
        #[cfg(not(feature = "debug"))]
        Some(&state.manifest),
        #[cfg(feature = "debug")]
        None,
    );
    debug!("Loaded css");

    let env = state.loader.acquire_env().unwrap();

    let templ = env.get_template("expenses.njk").unwrap();

    let res = match templ
        .render(context! { css => css, scripts => scripts, pages => PAGES, active => 1 })
    {
        Ok(templ) => templ,
        Err(e) => {
            let string = format!("{e}");
            tracing::error!(string);
            return Ok(Html(string));
        }
    };

    Ok(Html(res))
}

async fn get_receipt(PathParam(_id): PathParam<String>) -> axum::response::Result<Redirect> {
    Ok(Redirect::to("/"))
}

async fn new_form(State(state): State<Arc<AppState>>) -> axum::response::Result<Html<String>> {
    let scripts = assets::resolve_scripts(
        Path::new("client/main.ts"),
        #[cfg(not(feature = "debug"))]
        Some(&state.manifest),
        #[cfg(feature = "debug")]
        None,
    );
    debug!("Loaded scripts");

    let css = assets::resolve_css(
        Path::new("client/main.ts"),
        #[cfg(not(feature = "debug"))]
        Some(&state.manifest),
        #[cfg(feature = "debug")]
        None,
    );
    debug!("Loaded css");

    let env = state.loader.acquire_env().unwrap();

    let templ = env.get_template("new-expense.njk").unwrap();

    let res = match templ
        .render(context! { css => css, scripts => scripts, pages => PAGES, active => 0 })
    {
        Ok(templ) => templ,
        Err(e) => {
            let string = format!("{e}");
            tracing::error!(string);
            return Ok(Html(string));
        }
    };

    Ok(Html(res))
}
