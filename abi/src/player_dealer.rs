use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::{Amount, ChainId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Player {
    pub seat_id: u8, // single player: 0, multi player: 1-3
    pub bet: Amount,
    pub balance: Amount,
    pub hand: Vec<u8>,
    pub chain_id: Option<ChainId>,
    pub current_player: bool,
}

impl Player {
    pub fn new(seat_id: u8, balance: Amount, chain_id: ChainId) -> Self {
        Player {
            seat_id,
            bet: Amount::from_tokens(0),
            balance,
            hand: vec![],
            chain_id: Some(chain_id),
            current_player: false,
        }
    }

    pub fn add_bet(&mut self, amount: Amount, current_profile_balance: Amount) {
        if self.balance.ne(&current_profile_balance) {
            panic!("Profile and Player balance didn't match!");
        }

        let new_bet = self.bet.saturating_add(amount);
        if new_bet.gt(&self.balance) {
            panic!("Bets exceeding player balance!");
        }

        self.bet = new_bet
    }

    pub fn reset_bet(&mut self) {
        self.bet = Amount::from_tokens(0)
    }

    pub fn deal_bet(&mut self, min_bet: Amount, current_profile_balance: Amount) -> (Amount, Amount) {
        if self.balance.ne(&current_profile_balance) {
            panic!("Profile and Player balance didn't match!");
        }

        if min_bet.gt(&self.balance) {
            panic!("Minimum Bets exceeding player balance!");
        }

        if self.bet == Amount::ZERO {
            self.bet = min_bet
        }

        self.balance = self.balance.saturating_sub(self.bet);
        (self.bet, self.balance)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Dealer {
    pub hand: Vec<u8>,
}

impl Dealer {
    pub fn empty() -> Self {
        Dealer { hand: vec![] }
    }
}
