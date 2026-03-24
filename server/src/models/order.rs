use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn as_str(self) -> &'static str {
        match self {
            OrderSide::Bid => "bid",
            OrderSide::Ask => "ask",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub game_id: Uuid,
    pub player_id: Uuid,
    pub suit: String,
    pub side: String,
    pub price: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}
