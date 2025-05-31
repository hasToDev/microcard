use async_graphql_derive::{InputObject, SimpleObject};
use linera_sdk::linera_base_types::ChainId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject, InputObject)]
pub struct Player {
    pub seat_id: u8,
    pub bet: u64,
    pub balance: u64,
    pub hand: Vec<u8>,
    pub chain_id: ChainId,
    pub current_player: bool,
}

impl Player {
    pub fn new(seat_id: u8, balance: u64, chain_id: ChainId) -> Self {
        Player {
            seat_id,
            bet: 0,
            balance,
            hand: vec![],
            chain_id,
            current_player: false,
        }
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
