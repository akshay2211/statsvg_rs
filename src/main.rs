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
use params::{RawParams, Sections};

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

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("render") {
        render_once(&args[2..]).await;
        return;
    }

    serve().await;
}

async fn render_once(args: &[String]) {
    let mut user = String::new();
    let mut theme_name = String::from("github_dark");
    let mut sections = Sections {
        header: true,
        stats: true,
        grid: true,
        languages: true,
        top_repos: false,
    };
    let mut top_repos_count: usize = 3;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--user" | "--username" => {
                user = args.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--theme" => {
                theme_name = args.get(i + 1).cloned().unwrap_or_else(|| "github_dark".into());
                i += 2;
            }
            "--show-top-repos" => { sections.top_repos = true; i += 1; }
            "--no-header" => { sections.header = false; i += 1; }
            "--no-stats" => { sections.stats = false; i += 1; }
            "--no-grid" => { sections.grid = false; i += 1; }
            "--no-languages" => { sections.languages = false; i += 1; }
            "--top-repos-count" => {
                top_repos_count = args
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3)
                    .clamp(1, 6);
                i += 2;
            }
            other => {
                eprintln!("warn: unknown arg {other:?}");
                i += 1;
            }
        }
    }

    if user.is_empty() {
        eprintln!("error: --user <github-login> is required");
        std::process::exit(2);
    }

    let token = std::env::var("GH_TOKEN").unwrap_or_default();
    if token.is_empty() {
        warn!("GH_TOKEN not set — unauthenticated (60/hr per IP)");
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("failed to build HTTP client");

    let stats = match github::fetch_stats(&client, &token, &user).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error fetching stats for {user}: {e}");
            std::process::exit(1);
        }
    };

    let theme = theme::find_theme(&theme_name);
    let body = svg::render(&stats, theme, &sections, top_repos_count);
    print!("{body}");
}

async fn serve() {
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