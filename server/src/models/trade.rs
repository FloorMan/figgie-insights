use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Trade {
    pub id: Uuid,
    pub game_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub suit: String,
    pub price: i32,
    pub created_at: DateTime<Utc>,
}
