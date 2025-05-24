#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use abi::deck::Deck;
use async_graphql::{EmptySubscription, Object, Schema};
use linera_sdk::{graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service, ServiceRuntime};

use blackjack::BlackjackOperation;

use self::state::BlackjackState;

pub struct BlackjackService {
    state: Arc<BlackjackState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(BlackjackService);

impl WithServiceAbi for BlackjackService {
    type Abi = blackjack::BlackjackAbi;
}

impl Service for BlackjackService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = BlackjackState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        BlackjackService {
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
            BlackjackOperation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish()
        .execute(query)
        .await
    }
}

struct QueryRoot {
    state: Arc<BlackjackState>,
    runtime: Arc<ServiceRuntime<BlackjackService>>,
}

#[Object]
impl QueryRoot {
    async fn value(&self) -> &u64 {
        &self.state.value.get()
    }
    async fn get_deck(&self) -> Deck {
        self.state.deck_card.get().clone()
    }
}
