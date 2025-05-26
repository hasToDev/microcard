use crate::deck::Deck;
use crate::player_dealer::{Dealer, Player};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::ChannelName;
use serde::{Deserialize, Serialize};

/// Maximum number of players allowed in a Poker game.
const MAX_POKER_PLAYERS: usize = 8;

/// The channel name the application uses for cross-chain messages about game event.
const POKER_EVENT_NAME: &[u8] = b"poker";

pub fn poker_channel() -> ChannelName {
    ChannelName::from(POKER_EVENT_NAME.to_vec())
}

scalar!(BettingRound);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum BettingRound {
    PreFlop = 0,
    Flop = 1,
    Turn = 2,
    River = 3,
    Showdown = 4,
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct PokerGame {
    pub dealer: Dealer,
    pub players: Vec<Player>,
    pub deck: Deck,
    pub community_cards: Vec<u8>,
    pub pot: u64,
    pub current_round: BettingRound,
}

impl PokerGame {
    pub fn new(players: Vec<Player>) -> Result<Self, String> {
        if players.len() > MAX_POKER_PLAYERS {
            return Err(format!("Maximum of {} players allowed in Poker.", MAX_POKER_PLAYERS));
        }

        Ok(PokerGame {
            dealer: Dealer { hand: vec![] },
            players,
            deck: Deck::new(),
            community_cards: vec![],
            pot: 0,
            current_round: BettingRound::PreFlop,
        })
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        if self.players.len() >= MAX_POKER_PLAYERS {
            return Err(format!("Maximum of {} Poker players reached.", MAX_POKER_PLAYERS));
        }
        self.players.push(player);
        Ok(())
    }

    pub fn remove_player(&mut self, index: usize) -> Result<Player, String> {
        if index >= self.players.len() {
            return Err("Invalid Poker player index.".to_string());
        }
        Ok(self.players.remove(index))
    }
}
