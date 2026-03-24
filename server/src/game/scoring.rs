use crate::models::card::{Hand, Suit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerResult {
    pub player_id: Uuid,
    pub chips_start: i32,
    pub chips_end: i32,
    pub target_cards: u8,
    pub got_pot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub target_suit: Suit,
    pub common_suit: Suit,
    pub pot: i32,
    pub results: Vec<PlayerResult>,
}

/// Score a finished round.
///
/// Rules:
/// - Each player earns $10 × their target suit card count
/// - The player with the most target suit cards takes the pot ($200)
/// - Ties for majority: the pot is NOT awarded (stays in house) - standard Figgie rules
pub fn score_round(
    target_suit: Suit,
    common_suit: Suit,
    hands: &HashMap<Uuid, Hand>,
    chips: &HashMap<Uuid, i32>,
    pot: i32,
) -> ScoreResult {
    // Count target suit per player
    let target_counts: Vec<(Uuid, u8)> = hands
        .iter()
        .map(|(id, hand)| (*id, hand.count(target_suit)))
        .collect();

    // Find majority holder
    let max_count = target_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let majority_holders: Vec<Uuid> = target_counts
        .iter()
        .filter(|(_, c)| *c == max_count)
        .map(|(id, _)| *id)
        .collect();

    let pot_winner = if majority_holders.len() == 1 {
        Some(majority_holders[0])
    } else {
        None // tie: no one gets the pot
    };

    let mut results = Vec::new();
    for (player_id, target_cards) in &target_counts {
        let chips_start = *chips.get(player_id).unwrap_or(&50);
        let card_payout = (*target_cards as i32) * 10;
        let pot_bonus = if pot_winner == Some(*player_id) { pot } else { 0 };
        let chips_end = chips_start + card_payout + pot_bonus;

        results.push(PlayerResult {
            player_id: *player_id,
            chips_start,
            chips_end,
            target_cards: *target_cards,
            got_pot: pot_winner == Some(*player_id),
        });
    }

    ScoreResult {
        target_suit,
        common_suit,
        pot,
        results,
    }
}
