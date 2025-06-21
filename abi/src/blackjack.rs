use crate::bet_chip_profile::Profile;
use crate::deck::Deck;
use crate::player_dealer::{Dealer, Player};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::{ChannelName, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum number of players allowed in a Blackjack game.
pub const MAX_BLACKJACK_PLAYERS: usize = 3;

/// The channel name the application uses for cross-chain messages about game event.
const BLACKJACK_EVENT_NAME: &[u8] = b"blackjack";

pub fn blackjack_channel() -> ChannelName {
    ChannelName::from(BLACKJACK_EVENT_NAME.to_vec())
}

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
    pub pot: u64,
    pub active_seat: u8,
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
            pot: 0,
            active_seat: 0,
            status: BlackjackStatus::WaitingForPlayer,
            time_limit: Timestamp::from(0),
        }
    }

    pub fn is_seat_taken(&self, seat_id: u8) -> bool {
        self.players.contains_key(&seat_id)
    }

    pub fn register_player(&mut self, seat_id: u8, player: Player) {
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

    pub fn data_for_channel(&self) -> Self {
        BlackjackGame {
            sequence: self.sequence,
            dealer: Dealer::empty(), // TODO: hide dealer hand until BlackjackStatus::DealerTurn
            players: self.players.clone(),
            deck: Deck::empty(),
            pot: self.pot,
            active_seat: self.active_seat,
            status: self.status.clone(),
            time_limit: self.time_limit,
        }
    }
}
