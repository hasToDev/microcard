use async_graphql::{Request, Response, SimpleObject};
use linera_sdk::linera_base_types::{AccountOwner, Amount, Timestamp};
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BankrollParameters {
    pub bonus: Amount,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct DailyBonus {
    pub amount: Amount,
    pub last_claim: Timestamp,
}

impl DailyBonus {
    pub fn is_zero(&self) -> bool {
        self.amount == Amount::ZERO
    }
    pub fn update_bonus(&mut self, bonus: Amount) {
        if self.is_zero() {
            self.amount = bonus;
        }
    }
    pub fn claim_bonus(&mut self, current_time: Timestamp) -> Amount {
        let delta_since_last_claim = current_time.delta_since(self.last_claim).as_micros();
        if delta_since_last_claim >= ONE_DAY_CLAIM_DURATION_IN_MICROS {
            self.last_claim = current_time;
            return self.amount;
        }
        Amount::ZERO
    }
}

const ONE_DAY_CLAIM_DURATION_IN_MICROS: u64 = 60 * 60 * 24 * 1_000_000;
