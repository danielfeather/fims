use crate::{
    AppState, PAGES, assets, middleware,
    routes::expenses::types::{Currency, Frequency},
};
use axum::{
    Router,
    extract::{Multipart, Path as PathParam, State},
    response::{Html, Redirect},
    routing::get,
};
use chrono::NaiveDate;
use minijinja::context;
use serde::{
    Deserialize,
    de::{IntoDeserializer, value::StrDeserializer},
};
use std::{path::Path, sync::Arc};
use tower_sessions::Session;
use tracing::debug;

pub mod types;

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

    let css = assets::resolve_css(
        Path::new("client/main.ts"),
        #[cfg(not(feature = "debug"))]
        Some(&state.manifest),
        #[cfg(feature = "debug")]
        None,
    );

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

async fn post_form(session: Session, mut multipart: Multipart) -> axum::response::Result<Redirect> {
    let mut name: Option<String> = None;
    let mut currency: Option<Currency> = None;
    let mut amount: Option<u64> = None;
    let mut frequency: Frequency;
    let mut start_date: Option<NaiveDate> = None;
    let mut end_date: Option<NaiveDate> = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap().to_string();

        if field_name == "name" {
            name = Some(field.text().await?);
            continue;
        }

        if field_name == "currency" {
            let currency_raw = field.text().await?;

            let deserializer: StrDeserializer<serde::de::value::Error> =
                currency_raw.as_str().into_deserializer();

            currency = Some(Currency::deserialize(deserializer).map_err(|_| "Invalid currency")?);
            continue;
        }

        if field_name == "amount" {
            let number = u64::from_str_radix(field.text().await?.as_str(), 10)
                .map_err(|_| "Invalid amount")?;

            amount = Some(number);
            continue;
        }

        if field_name == "frequency" {
            let currency_raw = field.text().await?;

            let deserializer: StrDeserializer<serde::de::value::Error> =
                currency_raw.as_str().into_deserializer();

            frequency = Frequency::deserialize(deserializer).map_err(|_| "Invalid frequency")?;
            continue;
        }
    }

    Ok(Redirect::to("/upload/success"))
}
