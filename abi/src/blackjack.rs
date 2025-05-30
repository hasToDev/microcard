use crate::deck::Deck;
use crate::player_dealer::{Dealer, Player};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::ChannelName;
use serde::{Deserialize, Serialize};

/// Maximum number of players allowed in a Blackjack game.
pub const MAX_BLACKJACK_PLAYERS: usize = 3;

/// The channel name the application uses for cross-chain messages about game event.
const BLACKJACK_EVENT_NAME: &[u8] = b"blackjack";

pub fn blackjack_channel() -> ChannelName {
    ChannelName::from(BLACKJACK_EVENT_NAME.to_vec())
}

scalar!(BlackjackStatus);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum BlackjackStatus {
    WaitingForBets = 0,
    PlayerTurn = 1,
    DealerTurn = 2,
    Ended = 3,
}

scalar!(PlayChainStatus);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum PlayChainStatus {
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
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct BlackjackGame {
    pub dealer: Dealer,
    pub players: Vec<Player>,
    pub deck: Deck,
    pub pot: u64,
    pub status: BlackjackStatus,
}

impl BlackjackGame {
    pub fn new(players: Vec<Player>) -> Result<Self, String> {
        if players.len() > MAX_BLACKJACK_PLAYERS {
            return Err(format!("Maximum of {} players allowed in Blackjack.", MAX_BLACKJACK_PLAYERS));
        }

        Ok(BlackjackGame {
            dealer: Dealer { hand: vec![] },
            players,
            deck: Deck::new(),
            pot: 0,
            status: BlackjackStatus::WaitingForBets,
        })
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        if self.players.len() >= MAX_BLACKJACK_PLAYERS {
            return Err(format!("Maximum of {} Blackjack players reached.", MAX_BLACKJACK_PLAYERS));
        }
        self.players.push(player);
        Ok(())
    }

    pub fn remove_player(&mut self, index: usize) -> Result<Player, String> {
        if index >= self.players.len() {
            return Err("Invalid Blackjack player index.".to_string());
        }
        Ok(self.players.remove(index))
    }
}
