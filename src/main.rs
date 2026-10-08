#[cfg(not(feature = "debug"))]
use assets::Manifest;
use axum::{Router, extract::State, response::Html, routing::get};
use minijinja::{Environment, context, path_loader};
use minijinja_autoreload::AutoReloader;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::{path::Path, sync::Arc};
use tower_http::services::ServeDir;
use tower_sessions::{
    Expiry, MemoryStore, SessionManagerLayer,
    cookie::{Key, time::Duration},
};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

mod assets;
mod config;
mod middleware;
mod pricing;
mod routes;

const TEMPLATE_PATH: &str = "views";

pub struct AppState {
    loader: AutoReloader,
    #[cfg(not(feature = "debug"))]
    manifest: Manifest,
    _pool: Pool<Postgres>,
    config: Config,
    maintenance: bool,
}

const PAGES: &'static [(&str, &str)] = &[("Receipts", "/receipts"), ("Expenses", "/expenses")];

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::discover().await;

    #[allow(unused)]
    let maybe_manifest = assets::load_manifest();

    debug!("attempting to read DATABASE_URL");
    let database_url = std::env::var("DATABASE_URL")
        .expect("Unable to connect to DB, DATABASE_URL is not present in env");

    let _pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Unable to connect to DB");

    let key = Key::generate();

    let reloader = AutoReloader::new(|notifier| {
        let mut env = Environment::new();
        env.set_loader(path_loader(TEMPLATE_PATH));
        notifier.watch_path(TEMPLATE_PATH, true);
        Ok(env)
    });

    let session_store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_expiry(Expiry::OnInactivity(Duration::hours(12)))
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_signed(key);

    let listen_addr = format!("{}:{}", config.host, config.port);

    let state = Arc::new(AppState {
        loader: reloader,
        #[cfg(not(feature = "debug"))]
        manifest: maybe_manifest.expect("Unable to find asset manifest"),
        _pool,
        config,
        maintenance: std::env::var("MAINTENANCE").map_or(false, |val| val == "true"),
    });

    let app = Router::new()
        .route("/", get(index))
        .nest("/signin", routes::signin::router())
        .nest("/receipts", routes::receipts::router(state.clone()))
        .nest("/expenses", routes::expenses::router(state.clone()))
        .nest("/upload", routes::upload::router(state.clone()))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::maintenance,
        ))
        .layer(session_layer)
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let Ok(listener) = tokio::net::TcpListener::bind(&listen_addr).await else {
        error!("");
        return;
    };

    info!("Listening on {listen_addr}");

    axum::serve(listener, app).await.unwrap();
}

async fn index(State(state): State<Arc<AppState>>) -> axum::response::Result<Html<String>> {
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
    let env = state.loader.acquire_env().unwrap();

    let res = env
        .get_template("index.njk")
        .unwrap()
        .render(context! { css => css, scripts => scripts })
        .unwrap();

    Ok(Html(res))
}
