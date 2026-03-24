use crate::models::card::Suit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEntry {
    pub order_id: Uuid,
    pub player_id: Uuid,
    pub price: i32,
}

/// In-memory per-suit order book holding the single best bid and best ask.
/// Figgie uses a simple "last bid / last ask" market, not a full depth book.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuitBook {
    pub best_bid: Option<OrderEntry>,  // highest bid
    pub best_ask: Option<OrderEntry>,  // lowest ask
}

impl SuitBook {
    pub fn place_bid(&mut self, order_id: Uuid, player_id: Uuid, price: i32) {
        match &self.best_bid {
            Some(existing) if existing.price >= price => {} // existing is at least as good
            _ => {
                self.best_bid = Some(OrderEntry { order_id, player_id, price });
            }
        }
    }

    pub fn place_ask(&mut self, order_id: Uuid, player_id: Uuid, price: i32) {
        match &self.best_ask {
            Some(existing) if existing.price <= price => {} // existing is at least as good
            _ => {
                self.best_ask = Some(OrderEntry { order_id, player_id, price });
            }
        }
    }

    pub fn cancel_bid(&mut self, order_id: Uuid) {
        if let Some(bid) = &self.best_bid {
            if bid.order_id == order_id {
                self.best_bid = None;
            }
        }
    }

    pub fn cancel_ask(&mut self, order_id: Uuid) {
        if let Some(ask) = &self.best_ask {
            if ask.order_id == order_id {
                self.best_ask = None;
            }
        }
    }

    pub fn take_bid(&mut self) -> Option<OrderEntry> {
        self.best_bid.take()
    }

    pub fn take_ask(&mut self) -> Option<OrderEntry> {
        self.best_ask.take()
    }
}

/// Full order book across all 4 suits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderBook(pub HashMap<String, SuitBook>);

impl OrderBook {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        for suit in crate::models::card::Suit::ALL {
            map.insert(suit.as_str().to_string(), SuitBook::default());
        }
        OrderBook(map)
    }

    pub fn suit_mut(&mut self, suit: Suit) -> &mut SuitBook {
        self.0.entry(suit.as_str().to_string()).or_default()
    }

    pub fn suit(&self, suit: Suit) -> Option<&SuitBook> {
        self.0.get(suit.as_str())
    }
}
