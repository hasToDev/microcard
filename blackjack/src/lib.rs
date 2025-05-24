use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

use abi::player_dealer::Player;

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
    ResetAnalytics { p: Player },
    ShuffleCard { hash: String },
}
