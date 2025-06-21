#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Schema};
use bankroll::{BankrollOperation, DailyBonus};
use linera_sdk::{graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service, ServiceRuntime};

use self::state::BankrollState;

pub struct BankrollService {
    state: Arc<BankrollState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(BankrollService);

impl WithServiceAbi for BankrollService {
    type Abi = bankroll::BankrollAbi;
}

impl Service for BankrollService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = BankrollState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        BankrollService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, query: Self::Query) -> Self::QueryResponse {
        Schema::build(
            QueryRoot {
                state: self.state.clone(),
                runtime: self.runtime.clone(),
            },
            BankrollOperation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish()
        .execute(query)
        .await
    }
}

#[allow(dead_code)]
struct QueryRoot {
    state: Arc<BankrollState>,
    runtime: Arc<ServiceRuntime<BankrollService>>,
}

#[Object]
impl QueryRoot {
    async fn get_daily_bonus(&self) -> DailyBonus {
        self.state.daily_bonus.get().clone()
    }
}
