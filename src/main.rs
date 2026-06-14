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
use github::FetchOptions;
use params::{RawParams, RenderConfig, Variant};

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

    let config = params.config();
    let options = FetchOptions {
        include_lifetime: matches!(config.variant, Variant::Stats),
        fetch_avatar: matches!(config.variant, Variant::Profile) && config.sections.header,
    };

    let stats =
        github::fetch_stats(&state.client, &state.token, &params.username, &options).await?;
    let theme = theme::find_theme(&params.theme);
    let body = svg::render(&stats, theme, &config);

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
    let mut variant = Variant::Profile;
    let mut width: Option<f32> = None;
    let mut highlight: Option<String> = None;
    let mut top_repos_count: Option<usize> = None;
    let mut overrides: Vec<(String, bool)> = Vec::new();

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
            "--variant" => {
                variant = args
                    .get(i + 1)
                    .map(|s| Variant::parse(s))
                    .unwrap_or(Variant::Profile);
                i += 2;
            }
            "--width" => {
                width = args
                    .get(i + 1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .map(|w| w.clamp(360, 1600) as f32);
                i += 2;
            }
            "--highlight" => {
                highlight = args.get(i + 1).cloned().filter(|s| !s.is_empty());
                i += 2;
            }
            "--top-repos-count" => {
                top_repos_count = args
                    .get(i + 1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|n| n.clamp(1, 6));
                i += 2;
            }
            "--show-header"          => { overrides.push(("header".into(), true)); i += 1; }
            "--no-header"            => { overrides.push(("header".into(), false)); i += 1; }
            "--show-stats"           => { overrides.push(("stats".into(), true)); i += 1; }
            "--no-stats"             => { overrides.push(("stats".into(), false)); i += 1; }
            "--show-grid"            => { overrides.push(("grid".into(), true)); i += 1; }
            "--no-grid"              => { overrides.push(("grid".into(), false)); i += 1; }
            "--show-languages"       => { overrides.push(("languages".into(), true)); i += 1; }
            "--no-languages"         => { overrides.push(("languages".into(), false)); i += 1; }
            "--show-top-repos"       => { overrides.push(("top_repos".into(), true)); i += 1; }
            "--no-top-repos"         => { overrides.push(("top_repos".into(), false)); i += 1; }
            "--show-contributed-to"  => { overrides.push(("contributed_to".into(), true)); i += 1; }
            "--no-contributed-to"    => { overrides.push(("contributed_to".into(), false)); i += 1; }
            "--show-most-starred"    => { overrides.push(("most_starred".into(), true)); i += 1; }
            "--no-most-starred"      => { overrides.push(("most_starred".into(), false)); i += 1; }
            "--show-border"          => { overrides.push(("border".into(), true)); i += 1; }
            "--no-border"            => { overrides.push(("border".into(), false)); i += 1; }
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

    let mut config = RenderConfig::from_variant(variant);
    if let Some(w) = width { config.width = w; }
    if let Some(h) = highlight { config.highlight = Some(h); }
    if let Some(c) = top_repos_count { config.top_repos_count = c; }
    for (key, value) in overrides {
        let s = &mut config.sections;
        match key.as_str() {
            "header" => s.header = value,
            "stats" => s.stats = value,
            "grid" => s.grid = value,
            "languages" => s.languages = value,
            "top_repos" => s.top_repos = value,
            "contributed_to" => s.contributed_to = value,
            "most_starred" => s.most_starred = value,
            "border" => s.border = value,
            _ => {}
        }
    }

    let token = std::env::var("GH_TOKEN").unwrap_or_default();
    if token.is_empty() {
        warn!("GH_TOKEN not set — unauthenticated (60/hr per IP)");
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("failed to build HTTP client");

    let options = FetchOptions {
        include_lifetime: matches!(config.variant, Variant::Stats),
        fetch_avatar: matches!(config.variant, Variant::Profile) && config.sections.header,
    };

    let stats = match github::fetch_stats(&client, &token, &user, &options).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error fetching stats for {user}: {e}");
            std::process::exit(1);
        }
    };

    let theme = theme::find_theme(&theme_name);
    let body = svg::render(&stats, theme, &config);
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
