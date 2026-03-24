use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    Lobby,
    Active,
    Finished,
}

impl GameStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameStatus::Lobby => "lobby",
            GameStatus::Active => "active",
            GameStatus::Finished => "finished",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lobby" => Some(GameStatus::Lobby),
            "active" => Some(GameStatus::Active),
            "finished" => Some(GameStatus::Finished),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Game {
    pub id: Uuid,
    pub status: String,
    pub common_suit: Option<String>,
    pub target_suit: Option<String>,
    pub pot: i32,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GamePlayer {
    pub game_id: Uuid,
    pub player_id: Uuid,
    pub seat: i32,
    pub is_bot: bool,
    pub chips_start: i32,
    pub chips_end: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct GameWithPlayers {
    #[serde(flatten)]
    pub game: Game,
    pub players: Vec<GamePlayer>,
}
