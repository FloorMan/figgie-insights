use crate::game::engine::{GameView, TradeRecord};
use crate::game::scoring::ScoreResult;
use crate::models::card::{Hand, Suit};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent FROM the client (player) TO the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMsg {
    PlaceBid { suit: Suit, price: i32 },
    PlaceAsk { suit: Suit, price: i32 },
    CancelOrder { order_id: Uuid },
    AcceptBid { suit: Suit },
    AcceptAsk { suit: Suit },
    Pass,
    EndGame,  // host-only: trigger end of trading phase
}

/// Messages sent FROM the server TO clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMsg {
    /// Full public game state snapshot.
    GameState { game: GameView },
    /// A trade was executed.
    TradeExecuted { trade: TradeRecord },
    /// Sent to a joining player with their private hand.
    YourHand { hand: Hand },
    /// Game has started — hand is included (private, sent per-connection).
    GameStarted { your_hand: Hand, game: GameView },
    /// Game has ended with scoring results.
    GameEnded { result: ScoreResult },
    /// A player joined the lobby.
    PlayerJoined { player_id: Uuid, seat: usize },
    /// An order was placed (for all to see).
    OrderPlaced { order_id: Uuid, player_id: Uuid, suit: Suit, side: String, price: i32 },
    /// An order was cancelled.
    OrderCancelled { order_id: Uuid },
    /// Server error message.
    Error { message: String },
}
