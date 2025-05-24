use async_graphql_derive::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Default,
    Deserialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    SimpleObject,
    InputObject,
)]
pub struct Player {
    pub hand: Vec<u8>,
    pub balance: u64,
    pub bet: u64,
    pub active: bool,
}

#[derive(
    Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject,
)]
pub struct Dealer {
    pub hand: Vec<u8>,
}
