use abi::blackjack::BlackjackGame;
use async_graphql::{Request, Response};
use linera_sdk::linera_base_types::ChainId;
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub struct BlackjackAbi;

impl ContractAbi for BlackjackAbi {
    type Operation = BlackjackOperation;
    type Response = ();
}

impl ServiceAbi for BlackjackAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum BlackjackOperation {
    // * User Chain
    SubscribeTo { chain_id: ChainId },
    UnsubscribeFrom { chain_id: ChainId },
    ShuffleCard { hash: String },
    FindPlayChain {},
    RequestTableSeat { seat_id: u8 },
    // * Public Chain
    AddPlayChain { chain_id: ChainId },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BlackjackMessage {
    // * User Chain
    FindPlayChainResult { chain_id: Option<ChainId> },
    RequestTableSeatResult { seat_id: u8, success: bool },
    // * Play Chain
    Subscribe,
    Unsubscribe,
    RequestTableSeat { seat_id: u8, balance: u64 },
    // * Public Chain
    FindPlayChain,
    // * Channel Subscriber
    GameState { game: BlackjackGame },
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct BlackjackParameters {
    pub public_chains: Vec<ChainId>,
}
