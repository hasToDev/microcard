#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use self::state::BlackjackState;
use abi::bet_chip_profile::Profile;
use abi::blackjack::{GameData, UserStatus};
use abi::deck::Deck;
use async_graphql::{EmptySubscription, Object, Schema};
use blackjack::BlackjackOperation;
use linera_sdk::linera_base_types::ChainId;
use linera_sdk::{graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service, ServiceRuntime};

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
        let state = BlackjackState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
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

#[allow(dead_code)]
struct QueryRoot {
    state: Arc<BlackjackState>,
    runtime: Arc<ServiceRuntime<BlackjackService>>,
}

#[Object]
impl QueryRoot {
    async fn get_deck(&self) -> Deck {
        self.state.deck_card.get().clone()
    }
    async fn get_play_chains(&self) -> Vec<ChainId> {
        self.state.play_chain_status.indices().await.unwrap_or_default()
    }
    async fn single_player_data(&self) -> GameData {
        GameData {
            user_status: self.state.user_status.get().clone(),
            profile: self.state.profile.get().clone(),
            game: self.state.single_player_game.get().data_for_event(),
        }
    }
    async fn multi_player_data(&self) -> GameData {
        GameData {
            user_status: self.state.user_status.get().clone(),
            profile: self.state.profile.get().clone(),
            game: self.state.channel_game_state.get().data_for_event(),
        }
    }
    async fn get_profile(&self) -> Profile {
        self.state.profile.get().clone()
    }
    async fn get_user_status(&self) -> UserStatus {
        self.state.user_status.get().clone()
    }
}
