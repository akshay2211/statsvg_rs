mod error;
mod github;
mod params;
mod svg;
mod theme;

use std::{sync::Arc, time::Duration};

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use reqwest::Client;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use error::AppError;
use params::RawParams;

#[derive(Clone)]
struct AppState {
    client: Client,
    token: Arc<String>,
}

async fn stats_svg(
    State(state): State<AppState>,
    Query(params): Query<RawParams>,
) -> Result<impl IntoResponse, AppError> {
    info!(user = %params.username, theme = %params.theme, "request");

    let stats = github::fetch_stats(&state.client, &state.token, &params.username).await?;
    let theme = theme::find_theme(&params.theme);
    let sections = params.sections();
    let repo_count = params.top_repos_count();
    let body = svg::render(&stats, theme, &sections, repo_count);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=1800, stale-while-revalidate=3600"),
    );
    Ok((headers, body))
}

async fn list_themes() -> impl IntoResponse {
    axum::Json(
        theme::ALL_THEMES
            .iter()
            .map(|t| t.name)
            .collect::<Vec<_>>(),
    )
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("statsvg_rs=info")),
        )
        .init();

    let token = match std::env::var("GH_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => {
            warn!("GH_TOKEN not set — using unauthenticated requests (60/hr rate limit)");
            String::new()
        }
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client");

    let state = AppState {
        client,
        token: Arc::new(token),
    };

    let app = Router::new()
        .route("/api", get(stats_svg))
        .route("/themes", get(list_themes))
        .route("/health", get(health))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind 0.0.0.0:3000");
    info!("listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}