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
    pub fn empty() -> Self {
        Deck { cards: vec![] }
    }

    pub fn with_cards(cards: Vec<u8>) -> Self {
        Deck { cards }
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

    pub fn add_cards(&mut self, new_set: &mut Vec<u8>, timestamp: String) {
        self.cards.append(new_set);
        self.cards
            .shuffle(&mut get_custom_rng(timestamp.clone(), timestamp).expect("Failed to get custom rng").clone());
    }
}

pub fn get_new_deck(timestamp: String) -> Vec<u8> {
    let mut new_deck = Vec::from(CARD_DECKS);
    new_deck.shuffle(&mut get_custom_rng(timestamp.clone(), timestamp).expect("Failed to get custom rng").clone());
    new_deck
}

/// Calculate the total value of a blackjack hand.
///
/// # Card Values:
/// - Aces (1, 14, 27, 40): Counted as 11 or 1, whichever is better
/// - Face cards (Jack, Queen, King): 10
/// - Number cards (2-10): Their rank value
///
/// # Ace Handling:
/// Aces are initially counted as 11. If the total exceeds 21,
/// aces are converted to 1 until the hand is valid or all aces are adjusted.
pub fn calculate_hand_value(hand: &Vec<u8>) -> u8 {
    let mut total = 0u8;
    let mut aces = 0u8;

    for &card in hand {
        let rank = ((card - 1) % 13) + 1; // Get rank 1-13 for each suit
        match rank {
            1 => {
                // Ace
                aces += 1;
                total = total.saturating_add(11);
            }
            11 | 12 | 13 => {
                // Jack, Queen, King
                total = total.saturating_add(10);
            }
            _ => {
                // Number cards 2-10
                total = total.saturating_add(rank);
            }
        }
    }

    // Adjust for Aces if total exceeds 21
    while total > 21 && aces > 0 {
        total = total.saturating_sub(10); // Convert Ace from 11 to 1
        aces -= 1;
    }

    total
}
