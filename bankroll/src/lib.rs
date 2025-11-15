use async_graphql::scalar;
use async_graphql::{Request, Response, SimpleObject};
use linera_sdk::linera_base_types::{AccountOwner, Amount, ChainId, Timestamp};
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
    // * User Chain
    Balance { owner: AccountOwner },
    UpdateBalance { owner: AccountOwner, amount: Amount },
    NotifyDebt { amount: Amount, target_chain: ChainId },
    TransferTokenPot { amount: Amount, target_chain: ChainId },
    // * Master Chain
    MintToken { chain_id: ChainId, amount: Amount },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BankrollMessage {
    // * Public Chain
    TokenIssued { amount: Amount },
    DebtNotif { debt_id: u64, amount: Amount, created_at: Timestamp },
    TokenPot { amount: Amount },
    // * User Chain
    DebtPaid { debt_id: u64, amount: Amount, paid_at: Timestamp },
    // * Master Chain
    TokenUpdate { amount: Amount },
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum BankrollResponse {
    #[default]
    Ok,
    Balance(Amount),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BankrollParameters {
    pub master_chain: ChainId,
    pub bonus: Amount,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct DailyBonus {
    pub amount: Amount,
    pub last_claim: Timestamp,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PublicChainBalances {
    pub chain: ChainId,
    pub amount: Amount,
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

scalar!(DebtStatus);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum DebtStatus {
    Pending = 0,
    Paid = 1,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct DebtRecord {
    pub id: u64,
    pub user_chain: ChainId,
    pub amount: Amount,
    pub created_at: Timestamp,
    pub paid_at: Option<Timestamp>,
    pub status: DebtStatus,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct TokenPotRecord {
    pub id: u64,
    pub user_chain: ChainId,
    pub amount: Amount,
    pub created_at: Timestamp,
}

const ONE_DAY_CLAIM_DURATION_IN_MICROS: u64 = 60 * 60 * 24 * 1_000_000;
