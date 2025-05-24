use crate::deck::Deck;
use crate::player_dealer::{Dealer, Player};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use serde::{Deserialize, Serialize};

const MAX_BLACKJACK_PLAYERS: usize = 3;

scalar!(BlackjackStatus);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum BlackjackStatus {
    WaitingForBets = 0,
    PlayerTurn = 1,
    DealerTurn = 2,
    Ended = 3,
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
