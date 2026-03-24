use crate::game::engine::GameEngine;
use crate::ws::messages::ServerMsg;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub const BROADCAST_CAPACITY: usize = 64;

/// Shared state for a single game room held in memory.
pub struct GameRoom {
    pub engine: Arc<tokio::sync::Mutex<GameEngine>>,
    pub broadcast: broadcast::Sender<ServerMsg>,
}

impl GameRoom {
    pub fn new(engine: GameEngine) -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            engine: Arc::new(tokio::sync::Mutex::new(engine)),
            broadcast: tx,
        }
    }
}

/// Global application state, cloned cheaply via Arc internals.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub rooms: Arc<RwLock<HashMap<Uuid, GameRoom>>>,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
