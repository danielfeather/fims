use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{ErrorResponse, Redirect},
    routing::get,
};
use jsonwebtoken::{DecodingKey, Validation, jwk::JwkSet};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::error;
use url::Url;
use uuid::{Uuid, Version};

use crate::{AppState, config::IdentityProvider};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(redirect))
        .route("/callback", get(callback))
}

async fn redirect(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> axum::response::Result<Redirect> {
    let Some(idp_config) = state.config.identity_providers.get("default") else {
        return Err(StatusCode::SERVICE_UNAVAILABLE.into());
    };

    let IdentityProvider::Oidc(oidc) = &idp_config;

    let mut url = oidc.authorize_endpoint.clone();

    let redirect_uri = state
        .config
        .base_url
        .join("/signin/callback")
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let state = Uuid::now_v7().to_string();

    {
        let mut query = url.query_pairs_mut();
        query.append_pair("client_id", &oidc.client_id);
        query.append_pair("state", &state);
        query.append_pair("redirect_uri", redirect_uri.as_str());
        query.append_pair("scope", "openid");
    }

    let _ = session.insert("state", state);

    Ok(Redirect::to(url.as_str()))
}

#[derive(Debug, Deserialize)]
struct OAuthQuery {
    code: String,
    state: Uuid,
}

#[derive(Debug, Serialize)]
struct OAuthTokenRequest<'a> {
    grant_type: OAuthGrantType,
    code: &'a str,
    redirect_uri: Url,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum OAuthGrantType {
    AuthorizationCode,
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct IDTokenClaims {
    sub: String,
}

fn session_error<E>(_err: E) -> ErrorResponse {
    StatusCode::INTERNAL_SERVER_ERROR.into()
}

async fn callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OAuthQuery>,
    session: Session,
) -> axum::response::Result<Redirect> {
    if !query
        .state
        .get_version()
        .is_some_and(|v| matches!(v, Version::SortRand))
    {
        return Err(Redirect::to("/").into());
    }

    let Some(state_timestamp) = query.state.get_timestamp() else {
        return Err(Redirect::to("/").into());
    };

    // Should probably check this on app start up rather than allowing a non unix clock.
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time is set befire UNUX epoch");
    let hour = Duration::from_hours(1);
    let iat = Duration::from_secs(state_timestamp.to_unix().0);

    if (iat + hour) < now {
        return Err(Redirect::to("/signin").into());
    }

    let Some(idp_config) = state.config.identity_providers.get("default") else {
        return Err(StatusCode::SERVICE_UNAVAILABLE.into());
    };

    let IdentityProvider::Oidc(oidc) = &idp_config;

    let code = query.code;

    let client = reqwest::Client::new();

    let body = OAuthTokenRequest {
        grant_type: OAuthGrantType::AuthorizationCode,
        redirect_uri: state
            .config
            .base_url
            .join("/signin/callback")
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?,
        code: &code,
    };

    let res = client
        .post(oidc.token_endpoint.as_str())
        .form(&body)
        .basic_auth(&oidc.client_id, Some(&oidc.client_secret))
        .send()
        .await
        .map_err(|e| {
            error!("Token Request error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let token_res = res.json::<OAuthTokenResponse>().await.map_err(|e| {
        error!("deserialization error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let id_token = token_res.id_token;

    let header = jsonwebtoken::decode_header(id_token.as_bytes()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to decode id_token header",
        )
    })?;

    let Some(kid) = header.kid else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "No id token kid").into());
    };

    let jwks_uri = oidc
        .issuer
        .join("/.well-known/jwks.json")
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let jwks = reqwest::get(jwks_uri.as_str())
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get jwks"))?
        .json::<JwkSet>()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to deserialize jwks",
            )
        })?;

    let Some(jwk) = jwks.find(&kid) else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Unable to find jwk").into());
    };

    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unable to create decoding key: {e}"),
        )
    })?;

    let mut validation = Validation::new_for_family(jsonwebtoken::AlgorithmFamily::Rsa);

    let issuer = oidc.issuer.as_str();

    validation.set_audience(&[&oidc.client_id]);
    validation.set_issuer(&[&issuer[0..issuer.len() - 1]]);

    let token = jsonwebtoken::decode::<IDTokenClaims>(&id_token, &decoding_key, &validation)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = session
        .insert("user", token.claims.sub)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Unable to set session"));

    Ok(Redirect::to("/receipts"))
}
