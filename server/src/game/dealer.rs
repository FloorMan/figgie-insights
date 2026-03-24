use crate::models::card::{Hand, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use uuid::Uuid;

pub struct DealResult {
    pub common_suit: Suit,
    pub target_suit: Suit,
    /// Map from player_id (by seat index 0-3) to their hand
    pub hands: HashMap<Uuid, Hand>,
}

/// Build the 40-card Figgie deck and deal to 4 players.
///
/// Deck composition (total 40):
///   common suit : 12 cards
///   target suit (cross of common): 10 cards
///   other two suits: 8 cards each
pub fn deal(player_ids: &[Uuid; 4]) -> DealResult {
    let mut rng = thread_rng();

    // Pick common suit randomly
    let common_suit = *Suit::ALL.choose(&mut rng).unwrap();
    let target_suit = common_suit.cross_suit();

    // Build deck
    let mut deck: Vec<Suit> = Vec::with_capacity(40);
    for suit in Suit::ALL {
        let count = if suit == common_suit {
            12
        } else if suit == target_suit {
            10
        } else {
            8
        };
        for _ in 0..count {
            deck.push(suit);
        }
    }

    // Shuffle
    deck.shuffle(&mut rng);

    // Deal 10 cards to each of 4 players
    let mut hands: HashMap<Uuid, Hand> = HashMap::new();
    for (seat, player_id) in player_ids.iter().enumerate() {
        let hand = &mut hands.entry(*player_id).or_default();
        for card in &deck[seat * 10..(seat + 1) * 10] {
            hand.add(*card);
        }
    }

    DealResult {
        common_suit,
        target_suit,
        hands,
    }
}
