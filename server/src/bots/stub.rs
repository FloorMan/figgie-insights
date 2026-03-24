use crate::game::engine::GameView;
use crate::models::card::Suit;
use crate::ws::messages::ClientMsg;
use async_trait::async_trait;
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

/// Trait that all bot implementations must satisfy.
#[async_trait]
pub trait BotPlayer: Send + Sync {
    fn player_id(&self) -> Uuid;
    async fn take_action(&self, view: &GameView, my_hand: &crate::models::card::Hand) -> ClientMsg;
}

/// A bot that randomly places bids, asks, or passes.
pub struct RandomBot {
    pub id: Uuid,
}

impl RandomBot {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[async_trait]
impl BotPlayer for RandomBot {
    fn player_id(&self) -> Uuid {
        self.id
    }

    async fn take_action(&self, _view: &GameView, my_hand: &crate::models::card::Hand) -> ClientMsg {
        let mut rng = rand::thread_rng();
        let action: u8 = rng.gen_range(0..3);

        match action {
            0 => {
                // Random bid on a random suit
                let suit = *Suit::ALL.choose(&mut rng).unwrap();
                let price = rng.gen_range(1..=15);
                ClientMsg::PlaceBid { suit, price }
            }
            1 => {
                // Ask on a suit we hold
                let held: Vec<Suit> = Suit::ALL
                    .iter()
                    .filter(|&&s| my_hand.count(s) > 0)
                    .copied()
                    .collect();
                if held.is_empty() {
                    ClientMsg::Pass
                } else {
                    let suit = *held.choose(&mut rng).unwrap();
                    let price = rng.gen_range(1..=15);
                    ClientMsg::PlaceAsk { suit, price }
                }
            }
            _ => ClientMsg::Pass,
        }
    }
}
