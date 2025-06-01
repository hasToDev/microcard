use async_graphql::{Request, Response};
use linera_sdk::linera_base_types::{AccountOwner, Amount};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BankrollAbi;

impl ContractAbi for BankrollAbi {
    type Operation = BankrollOperation;
    type Response = BankrollResponse;
}

impl ServiceAbi for BankrollAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum BankrollOperation {
    Balance { owner: AccountOwner },
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum BankrollResponse {
    #[default]
    Ok,
    Balance(Amount),
}
