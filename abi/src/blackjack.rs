use crate::bet_chip_profile::Profile;
use crate::deck::Deck;
use crate::player_dealer::{Dealer, Player};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::{Amount, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum number of players allowed in a Blackjack game.
pub const MAX_BLACKJACK_PLAYERS: usize = 3;

/// The stream name the application uses for events about blackjack game event.
pub const BLACKJACK_STREAM_NAME: &[u8] = b"blackjack";

scalar!(BlackjackStatus);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum BlackjackStatus {
    #[default]
    WaitingForPlayer = 0,
    WaitingForBets = 1,
    PlayerTurn = 2,
    DealerTurn = 3,
    Ended = 4,
}

scalar!(MutationReason);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum MutationReason {
    AddNew = 0,
    Update = 1,
}

scalar!(UserStatus);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum UserStatus {
    #[default]
    Idle = 0,
    FindPlayChain = 1,
    PlayChainFound = 2,
    PlayChainUnavailable = 3,
    RequestingTableSeat = 4,
    RequestTableSeatFail = 5,
    InMultiPlayerGame = 6,
    InSinglePlayerGame = 7,
}

scalar!(GameOutcome);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum GameOutcome {
    PlayerWins = 0,
    DealerWins = 1,
    Draw = 3,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct GameData {
    pub profile: Profile,
    pub game: BlackjackGame,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct BlackjackGame {
    pub sequence: u64,
    pub dealer: Dealer,
    pub players: HashMap<u8, Player>,
    pub deck: Deck,
    pub pot: Amount,
    pub active_seat: u8, // single player: 0, multi player: 1-3
    pub status: BlackjackStatus,
    pub time_limit: Timestamp,
}

impl BlackjackGame {
    pub fn new(new_deck: Deck) -> Self {
        BlackjackGame {
            sequence: 0,
            dealer: Dealer { hand: vec![] },
            players: HashMap::new(),
            deck: new_deck,
            pot: Amount::from_tokens(0),
            active_seat: 0,
            status: BlackjackStatus::WaitingForPlayer,
            time_limit: Timestamp::from(0),
        }
    }

    pub fn is_seat_taken(&self, seat_id: u8) -> bool {
        self.players.contains_key(&seat_id)
    }

    pub fn register_update_player(&mut self, seat_id: u8, player: Player) {
        self.players.insert(seat_id, player);
    }

    pub fn update_status(&mut self, new_status: BlackjackStatus) {
        self.status = new_status;
    }

    pub fn remove_player(&mut self, seat_id: u8) {
        if self.players.contains_key(&seat_id) {
            self.players.remove(&seat_id).unwrap();
        }
    }

    pub fn draw_initial_cards(&mut self, seat_id: u8) {
        // Deal 2 cards to the dealer
        for _ in 0..2 {
            if let Some(card) = self.deck.deal() {
                self.dealer.hand.push(card);
            } else {
                panic!("Deck ran out of cards while dealing to dealer");
            }
        }

        // Get the player and deal 2 cards to them
        if let Some(player) = self.players.get_mut(&seat_id) {
            for _ in 0..2 {
                if let Some(card) = self.deck.deal() {
                    player.hand.push(card);
                } else {
                    panic!("Deck ran out of cards while dealing to player");
                }
            }
        } else {
            panic!("Player not found at seat {}", seat_id);
        }
    }

    pub fn data_for_event(&self) -> Self {
        // hide dealer hand until BlackjackStatus::Ended
        // TODO: enable this later, being disabled for debug
        // if self.status != BlackjackStatus::Ended {
        //     return BlackjackGame {
        //         sequence: self.sequence,
        //         dealer: Dealer::empty(),
        //         players: self.players.clone(),
        //         deck: Deck::empty(),
        //         pot: self.pot,
        //         active_seat: self.active_seat,
        //         status: self.status.clone(),
        //         time_limit: self.time_limit,
        //     };
        // }

        BlackjackGame {
            sequence: self.sequence,
            dealer: self.dealer.clone(),
            players: self.players.clone(),
            deck: Deck::empty(),
            pot: self.pot,
            active_seat: self.active_seat,
            status: self.status.clone(),
            time_limit: self.time_limit,
        }
    }
}
