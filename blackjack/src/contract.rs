#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BlackjackState;
use abi::blackjack::{blackjack_channel, PlayChainStatus, UserStatus, MAX_BLACKJACK_PLAYERS};
use abi::deck::Deck;
use abi::random::get_random_value;
use blackjack::{BlackjackMessage, BlackjackOperation, BlackjackParameters};
use linera_sdk::linera_base_types::{ChainId, MessageId};
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

pub struct BlackjackContract {
    state: BlackjackState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(BlackjackContract);

impl WithContractAbi for BlackjackContract {
    type Abi = blackjack::BlackjackAbi;
}

impl Contract for BlackjackContract {
    type Message = BlackjackMessage;
    type Parameters = BlackjackParameters;
    type InstantiationArgument = u64;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BlackjackState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        BlackjackContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        self.state.instantiate_value.set(argument);

        // validate that the application parameters were configured correctly.
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            // * User Chain
            BlackjackOperation::SubscribeTo { chain_id } => {
                self.message_manager(chain_id, BlackjackMessage::Subscribe);
            }
            BlackjackOperation::UnsubscribeFrom { chain_id } => {
                self.message_manager(chain_id, BlackjackMessage::Unsubscribe);
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
            BlackjackOperation::FindPlayChain {} => {
                let chain_id = self.get_public_chain();
                self.state.user_status.set(UserStatus::FindPlayChain);
                self.state.find_play_chain_retry.set(0);
                self.message_manager(chain_id, BlackjackMessage::FindPlayChain);
            }
            // * Public Chain
            BlackjackOperation::AddPlayChain { chain_id } => {
                self.play_chain_manager(chain_id, 0, PlayChainStatus::AddNew).await;
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let message_id = self.runtime.message_id().expect("Message ID has to be available when executing a message");

        match message {
            // * User Chain
            BlackjackMessage::FindPlayChainResult { chain_id } => {
                self.process_find_play_chain_result(message_id, chain_id);
            }
            // * Play Chain
            BlackjackMessage::Subscribe => {
                self.runtime.subscribe(message_id.chain_id, blackjack_channel());
            }
            BlackjackMessage::Unsubscribe => {
                self.runtime.unsubscribe(message_id.chain_id, blackjack_channel());
            }
            // * Public Chain
            BlackjackMessage::FindPlayChain => {
                log::info!(
                    "\nFindPlayChain Request Accepted at {:?} from: {:?}\n",
                    self.runtime.chain_id(),
                    message_id.chain_id
                );

                let result = self.search_available_play_chain().await;
                self.message_manager(message_id.chain_id, BlackjackMessage::FindPlayChainResult { chain_id: result });
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl BlackjackContract {
    fn message_manager(&mut self, destination: ChainId, message: BlackjackMessage) {
        self.runtime.prepare_message(message).with_tracking().send_to(destination);
    }

    // * User Chain
    fn get_public_chain(&mut self) -> ChainId {
        let i = get_random_value(
            0,
            self.runtime.application_parameters().public_chains.len() as u8,
            self.runtime.system_time().to_string(),
            self.runtime.system_time().to_string(),
        )
        .unwrap_or(0);

        *self.runtime.application_parameters().public_chains.get(i as usize).unwrap_or_else(|| {
            panic!("unable to find public chain");
        })
    }
    fn process_find_play_chain_result(&mut self, message_id: MessageId, chain_id: Option<ChainId>) {
        if let Some(chain) = chain_id {
            log::info!(
                "\nFindPlayChain Result Received at {:?} from: {:?}\n",
                self.runtime.chain_id(),
                message_id.chain_id
            );
            log::info!("Available Chain ID {:?}", chain);
            self.state.user_status.set(UserStatus::PlayChainFound);
            self.state.find_play_chain_retry.set(0);
            self.state.user_play_chain.set(vec![chain]);
            return;
        }

        let retry_count = *self.state.find_play_chain_retry.get();
        if retry_count >= 3 {
            log::info!("FindPlayChain Result Received : No Chain ID found!");
            self.state.user_status.set(UserStatus::PlayChainUnavailable);
            self.state.find_play_chain_retry.set(0);
            self.state.user_play_chain.get_mut().clear();
            return;
        }

        log::info!("Retrying FindPlayChain!");
        let next_chain_id = self.get_public_chain();
        self.state.find_play_chain_retry.set(retry_count.saturating_add(1));
        self.message_manager(next_chain_id, BlackjackMessage::FindPlayChain);
    }
    // * Public Chain
    async fn search_available_play_chain(&mut self) -> Option<ChainId> {
        for player_number in 0..MAX_BLACKJACK_PLAYERS {
            // Safely check if the key in play_chain_set exists and the vector is non-empty
            if let Some(vec) = self.state.play_chain_set.get(&(player_number as u8)).await.unwrap_or_default() {
                log::info!("search_available_play_chain play_chain_set vec len is {:?}", vec.len());
                if !vec.is_empty() {
                    return vec.first().cloned();
                }
            }
        }
        None
    }
    async fn play_chain_manager(&mut self, chain_id: ChainId, player_number: u8, status: PlayChainStatus) {
        if status == PlayChainStatus::Update {
            // remove chain_id from the current play_chain_set state
            if let Some(old_state) = self.state.play_chain_status.get(&chain_id).await.unwrap_or_default() {
                let mut vec_data = self.state.play_chain_set.get(&old_state).await.unwrap_or_default().unwrap_or_default();
                vec_data.retain(|c| c != &chain_id);
                self.state.play_chain_set.insert(&old_state, vec_data).unwrap_or_else(|_| {
                    panic!("Failed to update Play Chain Set for {:?}", chain_id);
                });
            }
        }

        // add chain_id to the new play_chain_set state
        let mut vec_data = self.state.play_chain_set.get(&player_number).await.unwrap_or_default().unwrap_or_default();
        vec_data.push(chain_id);
        self.state.play_chain_set.insert(&player_number, vec_data).unwrap_or_else(|_| {
            panic!("Failed to update Play Chain Set for {:?}", chain_id);
        });

        // update chain_id status on the play_chain_status
        self.state.play_chain_status.insert(&chain_id, player_number).unwrap_or_else(|_| {
            panic!("Failed to update Play Chain Status for {:?}", chain_id);
        });
    }
}
