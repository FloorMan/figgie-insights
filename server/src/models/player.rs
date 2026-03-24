use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlayer {
    pub username: String,
}

/// Aggregated stats returned from game_players history.
#[derive(Debug, Serialize)]
pub struct PlayerStats {
    pub player_id: Uuid,
    pub username: String,
    pub games_played: i64,
    pub total_chips_won: i64,
    pub total_chips_lost: i64,
}
