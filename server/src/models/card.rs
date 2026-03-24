use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

    /// The cross-suit (target when self is common).
    /// Spades <-> Hearts, Diamonds <-> Clubs
    pub fn cross_suit(self) -> Suit {
        match self {
            Suit::Spades => Suit::Hearts,
            Suit::Hearts => Suit::Spades,
            Suit::Diamonds => Suit::Clubs,
            Suit::Clubs => Suit::Diamonds,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Suit::Spades => "Spades",
            Suit::Hearts => "Hearts",
            Suit::Diamonds => "Diamonds",
            Suit::Clubs => "Clubs",
        }
    }

    pub fn from_str(s: &str) -> Option<Suit> {
        match s {
            "Spades" => Some(Suit::Spades),
            "Hearts" => Some(Suit::Hearts),
            "Diamonds" => Some(Suit::Diamonds),
            "Clubs" => Some(Suit::Clubs),
            _ => None,
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A player's hand: count of each suit held.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hand {
    pub spades: u8,
    pub hearts: u8,
    pub diamonds: u8,
    pub clubs: u8,
}

impl Hand {
    pub fn count(&self, suit: Suit) -> u8 {
        match suit {
            Suit::Spades => self.spades,
            Suit::Hearts => self.hearts,
            Suit::Diamonds => self.diamonds,
            Suit::Clubs => self.clubs,
        }
    }

    pub fn add(&mut self, suit: Suit) {
        match suit {
            Suit::Spades => self.spades += 1,
            Suit::Hearts => self.hearts += 1,
            Suit::Diamonds => self.diamonds += 1,
            Suit::Clubs => self.clubs += 1,
        }
    }

    pub fn remove(&mut self, suit: Suit) -> bool {
        match suit {
            Suit::Spades if self.spades > 0 => { self.spades -= 1; true }
            Suit::Hearts if self.hearts > 0 => { self.hearts -= 1; true }
            Suit::Diamonds if self.diamonds > 0 => { self.diamonds -= 1; true }
            Suit::Clubs if self.clubs > 0 => { self.clubs -= 1; true }
            _ => false,
        }
    }

    pub fn total(&self) -> u8 {
        self.spades + self.hearts + self.diamonds + self.clubs
    }
}
