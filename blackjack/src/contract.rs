#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BlackjackState;
use abi::blackjack::{blackjack_channel, BlackjackStatus, MutationReason, UserStatus, MAX_BLACKJACK_PLAYERS};
use abi::deck::Deck;
use abi::player_dealer::Player;
use abi::random::get_random_value;
use bankroll::{BankrollOperation, BankrollResponse};
use blackjack::{BlackjackMessage, BlackjackOperation, BlackjackParameters};
use linera_sdk::linera_base_types::{Amount, ChainId, MessageId};
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
            BlackjackOperation::RequestTableSeat { seat_id } => {
                if self.state.user_play_chain.get().is_none() {
                    panic!("no Play Chain found");
                }

                if seat_id == 0 || seat_id > MAX_BLACKJACK_PLAYERS as u8 {
                    panic!("seat_id is invalid, can only be 1-{:?}", MAX_BLACKJACK_PLAYERS);
                }

                if self.state.user_status.get().eq(&UserStatus::RequestingTableSeat) {
                    panic!("still waiting response from previous RequestTableSeat");
                }

                if self.state.user_status.get().eq(&UserStatus::InMultiPlayerGame) {
                    panic!("user already in game, can't request new seat");
                }

                let balance = self.state.profile.get().balance;
                let play_chain_id = self.state.user_play_chain.get().unwrap();
                self.message_manager(play_chain_id, BlackjackMessage::RequestTableSeat { seat_id, balance });
                self.state.user_status.set(UserStatus::RequestingTableSeat);
            }
            BlackjackOperation::GetBalance {} => {
                log::info!("BlackjackOperation::GetBalance");
                let balance = self.get_bankroll_balance();
                log::info!("Current Balance is {:?}", balance);
            }
            BlackjackOperation::Bet { amount } => {
                log::info!("BlackjackOperation::Bet");
                match self.state.user_status.get() {
                    UserStatus::InMultiPlayerGame => {
                        self.multi_player_bet(amount).await;
                    }
                    UserStatus::InSinglePlayerGame => {
                        // TODO: implement single player bet
                    }
                    _ => {
                        panic!("Player not in any Single or MultiPlayerGame!");
                    }
                }
            }
            BlackjackOperation::Deal {} => {
                log::info!("BlackjackOperation::Deal");
                // TODO: implement deal for both single and multi player
            }
            // * Public Chain
            BlackjackOperation::AddPlayChain { chain_id } => {
                self.play_chain_manager(chain_id, 0, MutationReason::AddNew).await;
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let message_id = self.runtime.message_id().expect("Message ID has to be available when executing a message");

        match message {
            // * User Chain
            BlackjackMessage::FindPlayChainResult { chain_id } => {
                if self.process_find_play_chain_result(message_id, chain_id) {
                    let balance = self.get_bankroll_balance();
                    let profile = self.state.profile.get_mut();
                    profile.update_balance(balance);
                    profile.calculate_chipset();
                }
            }
            BlackjackMessage::RequestTableSeatResult { seat_id, success } => {
                if success {
                    self.add_user_to_new_game(seat_id);
                    log::info!("RequestTableSeatResult SUCCESS on {:?}!", message_id.chain_id);
                    return;
                }
                self.state.user_status.set(UserStatus::RequestTableSeatFail);
                log::info!("RequestTableSeatResult FAILED on {:?}", message_id.chain_id);
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
            // * Play Chain
            BlackjackMessage::Subscribe => {
                self.runtime.subscribe(message_id.chain_id, blackjack_channel());
                log::info!("\nUser {:?} subscribe to Play Chain {:?}\n", message_id.chain_id, self.runtime.chain_id());
            }
            BlackjackMessage::Unsubscribe => {
                self.runtime.unsubscribe(message_id.chain_id, blackjack_channel());
                log::info!("\nUser {:?} unsubscribe from Play Chain {:?}\n", message_id.chain_id, self.runtime.chain_id());
            }
            BlackjackMessage::RequestTableSeat { seat_id, balance } => {
                if self.request_table_seat_manager(seat_id, balance, message_id).is_some() {
                    let game = self.state.game.get();
                    self.channel_manager(BlackjackMessage::ChannelGameState { game: game.data_for_channel() })
                }
                log::info!(
                    "\nUser {:?} RequestTableSeat to Play Chain {:?}\n",
                    message_id.chain_id,
                    self.runtime.chain_id()
                );
            }
            // * Channel Subscriber
            BlackjackMessage::ChannelGameState { game } => {
                log::info!("\nUser {:?} received new game state:\n {:?}", self.runtime.chain_id(), game);
                self.state.channel_game_state.set(game);
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

    fn get_bankroll_balance(&mut self) -> Amount {
        let owner = self.runtime.application_id().into();
        let bankroll_app_id = self.runtime.application_parameters().bankroll;
        let response = self.runtime.call_application(true, bankroll_app_id, &BankrollOperation::Balance { owner });
        match response {
            BankrollResponse::Balance(balance) => balance,
            response => panic!("Unexpected response from Bankroll application: {response:?}"),
        }
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
    fn process_find_play_chain_result(&mut self, message_id: MessageId, chain_id: Option<ChainId>) -> bool {
        if let Some(chain) = chain_id {
            log::info!(
                "\nFindPlayChain Result Received at {:?} from: {:?}\n",
                self.runtime.chain_id(),
                message_id.chain_id
            );
            log::info!("Available Chain ID {:?}", chain);
            self.state.user_status.set(UserStatus::PlayChainFound);
            self.state.find_play_chain_retry.set(0);
            self.state.user_play_chain.set(Some(chain));
            self.message_manager(chain, BlackjackMessage::Subscribe);
            return true;
        }

        let retry_count = *self.state.find_play_chain_retry.get();
        if retry_count >= 3 {
            log::info!("FindPlayChain Result Received : No Chain ID found!");
            self.state.user_status.set(UserStatus::PlayChainUnavailable);
            self.state.find_play_chain_retry.set(0);
            self.state.user_play_chain.set(None);
            return false;
        }

        log::info!("Retrying FindPlayChain!");
        let next_chain_id = self.get_public_chain();
        self.state.find_play_chain_retry.set(retry_count.saturating_add(1));
        self.message_manager(next_chain_id, BlackjackMessage::FindPlayChain);
        false
    }
    fn add_user_to_new_game(&mut self, seat_id: u8) {
        let balance = self.state.profile.get().balance;
        let chain_id = self.runtime.chain_id();
        self.state
            .player_seat_map
            .insert(&seat_id, Player::new(seat_id, balance, chain_id))
            .unwrap_or_else(|_| {
                panic!("Failed to update Player Seat Map for {:?} on add_user_to_new_game", chain_id);
            });
        self.state.profile.get_mut().update_seat(seat_id);
        self.state.user_status.set(UserStatus::InMultiPlayerGame);
    }
    async fn multi_player_bet(&mut self, amount: Amount) {
        if self.state.channel_game_state.get().status.ne(&BlackjackStatus::WaitingForBets) {
            panic!("game in play, waiting for next hands");
        }
        if self.state.profile.get().chipset.is_none() {
            panic!("missing Chipset data for placing bet");
        }

        let user_profile = self.state.profile.get().clone();
        let chipset = user_profile.chipset.unwrap();

        if user_profile.seat.is_none() {
            panic!("missing Player Seat ID");
        }
        if user_profile.balance.eq(&Amount::ZERO) || user_profile.balance.lt(&chipset.min_bet) {
            panic!("not enough Player balance");
        }
        if amount.lt(&chipset.min_bet) {
            panic!("minimum bet is {:?}", chipset.min_bet);
        }
        if amount.gt(&chipset.max_bet) {
            panic!("maximum bet is {:?}", chipset.max_bet);
        }

        let seat_id = user_profile.seat.unwrap();

        let player = self
            .state
            .player_seat_map
            .get_mut(&seat_id)
            .await
            .unwrap_or_else(|_| {
                panic!("Player not found!");
            })
            .unwrap_or_else(|| {
                panic!("Player not found!");
            });

        player.update_bet(amount);
    }
    // * Play Chain
    fn channel_manager(&mut self, message: BlackjackMessage) {
        self.runtime.prepare_message(message).with_tracking().send_to(blackjack_channel());
    }
    fn request_table_seat_manager(&mut self, seat_id: u8, balance: Amount, message_id: MessageId) -> Option<()> {
        let game = self.state.game.get_mut();

        if game.is_seat_taken(seat_id) {
            self.message_manager(message_id.chain_id, BlackjackMessage::RequestTableSeatResult { seat_id, success: false });
            return None;
        }

        let player = Player::new(seat_id, balance, message_id.chain_id);
        game.register_player(seat_id, player);
        self.message_manager(message_id.chain_id, BlackjackMessage::RequestTableSeatResult { seat_id, success: true });
        Some(())
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
    async fn play_chain_manager(&mut self, chain_id: ChainId, player_number: u8, status: MutationReason) {
        if status == MutationReason::Update {
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
