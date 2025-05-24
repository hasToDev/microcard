#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use abi::deck::Deck;
use blackjack::BlackjackOperation;
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

use self::state::BlackjackState;

pub struct BlackjackContract {
    state: BlackjackState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(BlackjackContract);

impl WithContractAbi for BlackjackContract {
    type Abi = blackjack::BlackjackAbi;
}

impl Contract for BlackjackContract {
    type Message = ();
    type Parameters = ();
    type InstantiationArgument = u64;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BlackjackState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        BlackjackContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        // validate that the application parameters were configured correctly.
        self.runtime.application_parameters();
        self.state.value.set(argument);
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            BlackjackOperation::ResetAnalytics { p } => {
                //
            }
            BlackjackOperation::ShuffleCard { hash } => {
                let mut current_deck = self.state.deck_card.get_mut();
                if current_deck.is_empty() {
                    self.state.deck_card.set(Deck::new());
                    current_deck = self.state.deck_card.get_mut();
                    current_deck.shuffle(hash, self.runtime.system_time().to_string());
                    log::info!("\nNew Deck:\n{:?}", current_deck.cards);
                    return;
                }
                current_deck.shuffle(hash, self.runtime.system_time().to_string());
                log::info!("\nShuffle Deck:\n{:?}", current_deck.cards);
            }
        }
    }

    async fn execute_message(&mut self, _message: Self::Message) {}

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}
