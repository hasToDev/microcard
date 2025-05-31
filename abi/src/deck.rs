use crate::random::get_custom_rng;
use async_graphql_derive::SimpleObject;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

/// Spades:
/// 1 = Ace, 2-10 = Rank 2 - Rank 10,
/// 11 = Jack, 12 = Queen, 13 = King
///
/// Hearts:
/// 14 = Ace, 15-23 = Rank 2 - Rank 10,
/// 24 = Jack, 25 = Queen, 26 = King
///
/// Diamonds:
/// 27 = Ace, 28-36 = Rank 2 - Rank 10,
/// 37 = Jack, 38 = Queen, 39 = King
///
/// Clubs:
/// 40 = Ace, 41-49 = Rank 2 - Rank 10,
/// 50 = Jack, 51 = Queen, 52 = King
pub const CARD_DECKS: [u8; 52] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
    42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52,
];

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Deck {
    pub cards: Vec<u8>,
}

impl Deck {
    pub fn new() -> Self {
        Deck { cards: Vec::from(CARD_DECKS) }
    }

    pub fn empty() -> Self {
        Deck { cards: vec![] }
    }

    pub fn shuffle(&mut self, hash: String, timestamp: String) {
        self.cards
            .shuffle(&mut get_custom_rng(hash, timestamp).expect("Failed to get custom rng").clone());
    }

    pub fn deal(&mut self) -> Option<u8> {
        self.cards.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.len() == 0
    }
}
