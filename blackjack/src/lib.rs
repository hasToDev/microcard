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
    // * Operation on User Chain
    Subscribe { chain_id: ChainId },
    Unsubscribe { chain_id: ChainId },
    ShuffleCard { hash: String },
    FindPlayChain {},
    // * Operation on Public Chain
    AddPlayChain { chain_id: ChainId },
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum BlackjackMessage {
    Subscribe,
    Unsubscribe,
    FindPlayChain,
    FindPlayChainResult { chain_id: Option<ChainId> },
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct BlackjackParameters {
    pub public_chains: Vec<ChainId>,
}
