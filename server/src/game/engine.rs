use crate::game::dealer;
use crate::game::order_book::OrderBook;
use crate::game::scoring::{score_round, ScoreResult};
use crate::models::card::{Hand, Suit};
use crate::models::game::GameStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub suit: Suit,
    pub price: i32,
}

/// A snapshot of game state sent to players (no hidden info here;
/// hands are sent individually by the WS session layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameView {
    pub game_id: Uuid,
    pub status: GameStatus,
    pub players: Vec<Uuid>,          // in seat order
    pub chips: HashMap<Uuid, i32>,   // current chip counts
    pub order_book: OrderBook,
    pub trade_log: Vec<TradeRecord>,
    pub target_suit: Option<Suit>,   // revealed only after game ends
    pub common_suit: Option<Suit>,   // revealed only after game ends
}

/// Core in-memory game engine. Handles all game logic; persistence is done
/// by handlers that observe the engine's outputs.
#[derive(Debug)]
pub struct GameEngine {
    pub game_id: Uuid,
    pub status: GameStatus,
    /// Seat-ordered list of player UUIDs (index = seat number)
    pub players: Vec<Uuid>,
    pub is_bot: HashMap<Uuid, bool>,
    pub hands: HashMap<Uuid, Hand>,
    pub chips: HashMap<Uuid, i32>,
    pub order_book: OrderBook,
    pub trade_log: Vec<TradeRecord>,
    pub common_suit: Option<Suit>,
    pub target_suit: Option<Suit>,
    pub score_result: Option<ScoreResult>,
}

impl GameEngine {
    pub fn new(game_id: Uuid) -> Self {
        Self {
            game_id,
            status: GameStatus::Lobby,
            players: Vec::new(),
            is_bot: HashMap::new(),
            hands: HashMap::new(),
            chips: HashMap::new(),
            order_book: OrderBook::new(),
            trade_log: Vec::new(),
            common_suit: None,
            target_suit: None,
            score_result: None,
        }
    }

    /// Add a player (human or bot) to the lobby. Returns their seat index.
    pub fn add_player(&mut self, player_id: Uuid, is_bot: bool) -> Result<usize, String> {
        if self.status != GameStatus::Lobby {
            return Err("Game is not in lobby".into());
        }
        if self.players.len() >= 4 {
            return Err("Game is full".into());
        }
        if self.players.contains(&player_id) {
            return Err("Player already in game".into());
        }
        let seat = self.players.len();
        self.players.push(player_id);
        self.is_bot.insert(player_id, is_bot);
        self.chips.insert(player_id, 50);
        Ok(seat)
    }

    /// Start the game: deal cards, transition to Active.
    pub fn start(&mut self) -> Result<(), String> {
        if self.status != GameStatus::Lobby {
            return Err("Game is not in lobby".into());
        }
        if self.players.len() != 4 {
            return Err(format!("Need 4 players, have {}", self.players.len()));
        }

        let player_arr: [Uuid; 4] = self.players.as_slice().try_into()
            .map_err(|_| "Failed to convert players to array")?;
        let deal_result = dealer::deal(&player_arr);

        self.hands = deal_result.hands;
        self.common_suit = Some(deal_result.common_suit);
        self.target_suit = Some(deal_result.target_suit);
        self.order_book = OrderBook::new();
        self.status = GameStatus::Active;
        Ok(())
    }

    /// Place a bid (offer to buy) on a suit at a price.
    /// Returns the new order_id if successful.
    pub fn place_bid(
        &mut self,
        player_id: Uuid,
        suit: Suit,
        price: i32,
    ) -> Result<Uuid, String> {
        self.require_active()?;
        self.require_player(player_id)?;
        if price <= 0 {
            return Err("Price must be positive".into());
        }
        let order_id = Uuid::new_v4();
        self.order_book.suit_mut(suit).place_bid(order_id, player_id, price);
        Ok(order_id)
    }

    /// Place an ask (offer to sell) on a suit at a price.
    pub fn place_ask(
        &mut self,
        player_id: Uuid,
        suit: Suit,
        price: i32,
    ) -> Result<Uuid, String> {
        self.require_active()?;
        self.require_player(player_id)?;
        let hand = self.hands.get(&player_id).ok_or("Player has no hand")?;
        if hand.count(suit) == 0 {
            return Err(format!("No {} cards to sell", suit));
        }
        if price <= 0 {
            return Err("Price must be positive".into());
        }
        let order_id = Uuid::new_v4();
        self.order_book.suit_mut(suit).place_ask(order_id, player_id, price);
        Ok(order_id)
    }

    /// Accept the best ask for a suit (buy). Returns completed TradeRecord.
    pub fn accept_ask(
        &mut self,
        buyer_id: Uuid,
        suit: Suit,
    ) -> Result<TradeRecord, String> {
        self.require_active()?;
        self.require_player(buyer_id)?;

        let ask = self.order_book.suit_mut(suit)
            .take_ask()
            .ok_or("No ask to accept")?;

        if ask.player_id == buyer_id {
            // Put it back and error
            self.order_book.suit_mut(suit).place_ask(ask.order_id, ask.player_id, ask.price);
            return Err("Cannot trade with yourself".into());
        }

        self.execute_trade(buyer_id, ask.player_id, suit, ask.price)
    }

    /// Accept the best bid for a suit (sell). Returns completed TradeRecord.
    pub fn accept_bid(
        &mut self,
        seller_id: Uuid,
        suit: Suit,
    ) -> Result<TradeRecord, String> {
        self.require_active()?;
        self.require_player(seller_id)?;

        let bid = self.order_book.suit_mut(suit)
            .take_bid()
            .ok_or("No bid to accept")?;

        if bid.player_id == seller_id {
            self.order_book.suit_mut(suit).place_bid(bid.order_id, bid.player_id, bid.price);
            return Err("Cannot trade with yourself".into());
        }

        self.execute_trade(bid.player_id, seller_id, suit, bid.price)
    }

    fn execute_trade(
        &mut self,
        buyer_id: Uuid,
        seller_id: Uuid,
        suit: Suit,
        price: i32,
    ) -> Result<TradeRecord, String> {
        // Verify seller still has the card
        {
            let seller_hand = self.hands.get(&seller_id).ok_or("Seller has no hand")?;
            if seller_hand.count(suit) == 0 {
                return Err(format!("Seller has no {} to sell", suit));
            }
        }
        // Verify buyer has enough chips
        let buyer_chips = *self.chips.get(&buyer_id).unwrap_or(&0);
        if buyer_chips < price {
            return Err("Buyer has insufficient chips".into());
        }

        // Transfer card
        self.hands.get_mut(&seller_id).unwrap().remove(suit);
        self.hands.get_mut(&buyer_id).unwrap().add(suit);

        // Transfer chips
        *self.chips.get_mut(&buyer_id).unwrap() -= price;
        *self.chips.get_mut(&seller_id).unwrap() += price;

        let record = TradeRecord { buyer_id, seller_id, suit, price };
        self.trade_log.push(record.clone());
        Ok(record)
    }

    /// Cancel an order by ID (searches all suits).
    pub fn cancel_order(&mut self, player_id: Uuid, order_id: Uuid) {
        for suit in Suit::ALL {
            let book = self.order_book.suit_mut(suit);
            if let Some(bid) = &book.best_bid {
                if bid.order_id == order_id && bid.player_id == player_id {
                    book.cancel_bid(order_id);
                    return;
                }
            }
            if let Some(ask) = &book.best_ask {
                if ask.order_id == order_id && ask.player_id == player_id {
                    book.cancel_ask(order_id);
                    return;
                }
            }
        }
    }

    /// End the trading phase and compute final scores.
    pub fn end_game(&mut self) -> Result<ScoreResult, String> {
        if self.status != GameStatus::Active {
            return Err("Game is not active".into());
        }
        let target = self.target_suit.ok_or("No target suit set")?;
        let common = self.common_suit.ok_or("No common suit set")?;
        let result = score_round(target, common, &self.hands, &self.chips, 200);

        // Apply chip changes
        for pr in &result.results {
            self.chips.insert(pr.player_id, pr.chips_end);
        }

        self.status = GameStatus::Finished;
        self.score_result = Some(result.clone());
        Ok(result)
    }

    /// Build a public game view (hides suit identity until game over).
    pub fn view(&self) -> GameView {
        GameView {
            game_id: self.game_id,
            status: self.status.clone(),
            players: self.players.clone(),
            chips: self.chips.clone(),
            order_book: self.order_book.clone(),
            trade_log: self.trade_log.clone(),
            target_suit: if self.status == GameStatus::Finished { self.target_suit } else { None },
            common_suit: if self.status == GameStatus::Finished { self.common_suit } else { None },
        }
    }

    /// Get a player's private hand (only sent to that player).
    pub fn hand_for(&self, player_id: Uuid) -> Option<&Hand> {
        self.hands.get(&player_id)
    }

    fn require_active(&self) -> Result<(), String> {
        if self.status != GameStatus::Active {
            Err("Game is not active".into())
        } else {
            Ok(())
        }
    }

    fn require_player(&self, player_id: Uuid) -> Result<(), String> {
        if !self.players.contains(&player_id) {
            Err("Player not in this game".into())
        } else {
            Ok(())
        }
    }
}
