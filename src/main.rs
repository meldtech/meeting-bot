use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

#[derive(Clone, Default)]
struct AppState {
    /// Base URL of the Kair Voice API (no trailing slash). Used by future upload logic.
    kair_api_base_url: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct JoinRequest {
    bearer_token: String,
    url: String,
    name: String,
    team_id: String,
    timezone: String,
    user_id: String,
    bot_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JoinAccepted {
    status: &'static str,
    platform: &'static str,
    bot_id: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

fn validate_join(body: &JoinRequest) -> Result<(), ApiError> {
    if body.bearer_token.trim().is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "bearerToken must be non-empty".into(),
        });
    }
    Uuid::parse_str(body.bot_id.trim()).map_err(|_| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "botId must be a valid UUID (Kair session id)".into(),
    })?;
    Url::parse(body.url.trim()).map_err(|_| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "url must be a valid absolute URL".into(),
    })?;
    Ok(())
}

fn host_matches(url: &str, hosts: &[&str]) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    parsed.host_str().is_some_and(|h| {
        hosts
            .iter()
            .any(|suffix| h == *suffix || h.ends_with(&format!(".{suffix}")))
    })
}

fn validate_platform_url(platform: &'static str, meeting_url: &str) -> Result<(), ApiError> {
    let ok = match platform {
        "google_meet" => host_matches(meeting_url, &["meet.google.com"]),
        "zoom" => host_matches(meeting_url, &["zoom.us", "zoom.com"]),
        "microsoft_teams" => host_matches(meeting_url, &["teams.microsoft.com", "teams.live.com"]),
        "jitsi" => meeting_url.starts_with("http://") || meeting_url.starts_with("https://"),
        _ => false,
    };
    if !ok {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: format!("url does not look like a {platform} meeting link"),
        });
    }
    Ok(())
}

async fn join_handler(
    State(state): State<Arc<AppState>>,
    platform: &'static str,
    body: JoinRequest,
) -> Result<(StatusCode, Json<JoinAccepted>), ApiError> {
    validate_join(&body)?;
    validate_platform_url(platform, &body.url)?;

    info!(
        platform,
        bot_id = %body.bot_id,
        meeting_url = %body.url,
        display_name = %body.name,
        team_id = %body.team_id,
        timezone = %body.timezone,
        bot_user_id = %body.user_id,
        "join request accepted (automation not implemented in this stub)"
    );

    let bot_id = body.bot_id.clone();
    let kair_base = state.kair_api_base_url.clone();
    tokio::spawn(async move {
        stub_join_worker(platform, body, kair_base).await;
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(JoinAccepted {
            status: "accepted",
            platform,
            bot_id,
        }),
    ))
}

async fn join_google(
    state: State<Arc<AppState>>,
    Json(body): Json<JoinRequest>,
) -> Result<(StatusCode, Json<JoinAccepted>), ApiError> {
    join_handler(state, "google_meet", body).await
}

async fn join_zoom(
    state: State<Arc<AppState>>,
    Json(body): Json<JoinRequest>,
) -> Result<(StatusCode, Json<JoinAccepted>), ApiError> {
    join_handler(state, "zoom", body).await
}

async fn join_microsoft(
    state: State<Arc<AppState>>,
    Json(body): Json<JoinRequest>,
) -> Result<(StatusCode, Json<JoinAccepted>), ApiError> {
    join_handler(state, "microsoft_teams", body).await
}

async fn join_jitsi(
    state: State<Arc<AppState>>,
    Json(body): Json<JoinRequest>,
) -> Result<(StatusCode, Json<JoinAccepted>), ApiError> {
    join_handler(state, "jitsi", body).await
}

/// Placeholder for browser / RTC automation and `POST .../upload-audio` callbacks.
async fn stub_join_worker(platform: &'static str, body: JoinRequest, kair_base: Option<String>) {
    if let Some(base) = kair_base.as_ref() {
        let upload_target = format!("{base}/sessions/{}/upload-audio", body.bot_id);
        tracing::debug!(
            %platform,
            bot_id = %body.bot_id,
            kair_api_base_url = %base,
            %upload_target,
            "stub: upload target (not called)"
        );
    } else {
        tracing::debug!(
            %platform,
            bot_id = %body.bot_id,
            "stub: set KAIR_API_BASE_URL to log intended callback base"
        );
    }
}

async fn health() -> &'static str {
    "ok"
}

fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/google/join", post(join_google))
        .route("/zoom/join", post(join_zoom))
        .route("/microsoft/join", post(join_microsoft))
        .route("/jitsi/join", post(join_jitsi))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kair_meeting_bot=info,tower_http=info".into()),
        )
        .init();

    let host: String = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);

    let kair_api_base_url = std::env::var("KAIR_API_BASE_URL")
        .ok()
        .map(|s| s.trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty());

    let state = Arc::new(AppState { kair_api_base_url });
    let app = app(state);

    let addr: SocketAddr = format!("{host}:{port}").parse().expect("invalid HOST:PORT");
    info!(%addr, "kair-meeting-bot listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    if let Err(e) = axum::serve(listener, app).await {
        error!(%e, "server error");
    }
}
