mod bots;
mod config;
mod db;
mod error;
mod game;
mod handlers;
mod models;
mod state;
mod ws;

use crate::config::Config;
use crate::handlers::{
    games::{create_game, get_game, get_game_history, join_game, list_games},
    players::{create_player, get_player, get_player_stats},
    ws::ws_handler,
};
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Init tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "figgie_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    tracing::info!("Connecting to database...");
    let pool = db::create_pool(&config.database_url).await?;

    // Run migrations automatically on startup
    tracing::info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState::new(pool);

    let app = Router::new()
        // Player routes
        .route("/players", post(create_player))
        .route("/players/:id", get(get_player))
        .route("/players/:id/stats", get(get_player_stats))
        // Game routes
        .route("/games", post(create_game).get(list_games))
        .route("/games/:id", get(get_game))
        .route("/games/:id/join", post(join_game))
        .route("/games/:id/history", get(get_game_history))
        // WebSocket
        .route("/games/:id/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Figgie server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
