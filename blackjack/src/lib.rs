use abi::blackjack::BlackjackGame;
use async_graphql::{Request, Response};
use bankroll::BankrollAbi;
use linera_sdk::linera_base_types::{Amount, ApplicationId, ChainId};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
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
    FindPlayChain {},
    RequestTableSeat { seat_id: u8 },
    GetBalance {},
    Bet { amount: Amount },
    Deal {},
    StartSinglePlayerGame {},
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
    RequestTableSeat { seat_id: u8, balance: Amount },
    // * Public Chain
    FindPlayChain,
    // * Channel Subscriber
    ChannelGameState { game: BlackjackGame },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlackjackParameters {
    pub public_chains: Vec<ChainId>,
    pub bankroll: ApplicationId<BankrollAbi>,
}
