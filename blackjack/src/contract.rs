#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BlackjackState;
use abi::blackjack::{BlackjackGame, BlackjackStatus, MutationReason, UserStatus, BLACKJACK_STREAM_NAME, MAX_BLACKJACK_PLAYERS};
use abi::deck::{get_new_deck, Deck};
use abi::player_dealer::Player;
use abi::random::get_random_value;
use bankroll::{BankrollOperation, BankrollResponse};
use blackjack::{BlackjackEvent, BlackjackMessage, BlackjackOperation, BlackjackParameters};
use linera_sdk::linera_base_types::{Amount, ChainId, StreamUpdate};
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
    type EventValue = BlackjackEvent;

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
            BlackjackOperation::FindPlayChain {} => {
                // TODO: make UserStatus check to prevent double calling FindPlayChain
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
                        if self.state.channel_game_state.get().status.ne(&BlackjackStatus::WaitingForBets) {
                            panic!("game in play, not ready for placing bets, please wait for the next hands");
                        }
                        log::info!("Bet MultiPlayerGame");
                        self.player_bet(amount).await;
                    }
                    UserStatus::InSinglePlayerGame => {
                        if self.state.single_player_game.get().status.ne(&BlackjackStatus::WaitingForBets) {
                            panic!("game in play, not ready for placing bets, please wait for the next hands");
                        }
                        log::info!("Bet SinglePlayerGame");
                        self.player_bet(amount).await;
                    }
                    _ => {
                        panic!("Player not in any Single or MultiPlayerGame!");
                    }
                }
            }
            BlackjackOperation::Deal {} => {
                log::info!("BlackjackOperation::Deal");
                match self.state.user_status.get() {
                    UserStatus::InMultiPlayerGame => {
                        panic!("multi player deal not implemented yet");
                        // TODO: implement deal for multi player
                        // TODO: after the last player call DEAL, continue with changing BlackjackStatus,drawing card for both dealer and players
                        // if self.state.channel_game_state.get().status.ne(&BlackjackStatus::WaitingForBets) {
                        //     panic!("game in play, not ready for dealing bets, please wait for the next hands");
                        // }
                        // log::info!("Deal MultiPlayerGame");
                    }
                    UserStatus::InSinglePlayerGame => {
                        if self.state.single_player_game.get().status.ne(&BlackjackStatus::WaitingForBets) {
                            panic!("game in play, not ready for dealing bets, please wait for the next hands");
                        }
                        log::info!("Deal SinglePlayerGame");
                        self.deal_single_player().await;
                        self.draw_single_player().await;
                    }
                    _ => {
                        panic!("Player not in any Single or MultiPlayerGame!");
                    }
                }
            }
            BlackjackOperation::StartSinglePlayerGame {} => {
                log::info!("BlackjackOperation::StartSinglePlayerGame");
                match self.state.user_status.get() {
                    UserStatus::Idle | UserStatus::PlayChainUnavailable => {
                        self.update_profile_balance_and_bet_data();
                        self.add_user_to_new_single_player_game();
                        let token_pool_address = self.get_public_chain();
                        self.state.token_pool_address.set(Some(token_pool_address));
                    }
                    current_status => {
                        panic!("Unable to Start Single Player Game, user status is {:?}", current_status);
                    }
                }
            }
            // * Master Chain
            BlackjackOperation::AddPlayChain {
                target_public_chain,
                play_chain_id,
            } => {
                assert_eq!(
                    self.runtime.chain_id(),
                    self.runtime.application_parameters().master_chain,
                    "MasterChain Authorization Required for BankrollOperation::AddPlayChain"
                );
                log::info!("BlackjackOperation::AddPlayChain at {:?}", self.runtime.authenticated_signer());
                self.message_manager(target_public_chain, BlackjackMessage::AddPlayChain { chain_id: play_chain_id });
            }
            BlackjackOperation::MintToken { chain_id, amount } => {
                assert_eq!(
                    self.runtime.chain_id(),
                    self.runtime.application_parameters().master_chain,
                    "MasterChain Authorization Required for BlackjackOperation::MintToken"
                );
                log::info!("BlackjackOperation::MintToken at {:?}", self.runtime.authenticated_signer());
                let bankroll_app_id = self.runtime.application_parameters().bankroll;
                self.runtime
                    .call_application(true, bankroll_app_id, &BankrollOperation::MintToken { chain_id, amount });
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let origin_chain_id = self.runtime.message_origin_chain_id().expect("Chain ID missing from message");

        match message {
            // * User Chain
            BlackjackMessage::FindPlayChainResult { chain_id } => {
                if self.process_find_play_chain_result(origin_chain_id, chain_id) {
                    self.update_profile_balance_and_bet_data();
                }
            }
            BlackjackMessage::RequestTableSeatResult { seat_id, success } => {
                if success {
                    self.add_user_to_new_multi_player_game(seat_id);
                    log::info!("RequestTableSeatResult SUCCESS on {:?}!", origin_chain_id);
                    return;
                }
                self.state.user_status.set(UserStatus::RequestTableSeatFail);
                log::info!("RequestTableSeatResult FAILED on {:?}", origin_chain_id);
            }
            // * Public Chain
            BlackjackMessage::FindPlayChain => {
                log::info!(
                    "\nFindPlayChain Request Accepted at {:?} from: {:?}\n",
                    self.runtime.chain_id(),
                    origin_chain_id
                );

                let result = self.search_available_play_chain().await;
                self.message_manager(origin_chain_id, BlackjackMessage::FindPlayChainResult { chain_id: result });
            }
            BlackjackMessage::AddPlayChain { chain_id } => {
                assert_eq!(
                    origin_chain_id,
                    self.runtime.application_parameters().master_chain,
                    "MasterChain Authorization Required for BlackjackMessage::AddPlayChain"
                );
                log::info!("BankrollMessage::AddPlayChain from {:?} at {:?}", origin_chain_id, self.runtime.chain_id());
                self.play_chain_manager(chain_id, 0, MutationReason::AddNew).await;
            }
            // * Play Chain
            BlackjackMessage::Subscribe => {
                let app_id = self.runtime.application_id().forget_abi();
                self.runtime.subscribe_to_events(origin_chain_id, app_id, BLACKJACK_STREAM_NAME.into());
                log::info!("\nUser {:?} subscribe to Play Chain {:?}\n", origin_chain_id, self.runtime.chain_id());
            }
            BlackjackMessage::Unsubscribe => {
                let app_id = self.runtime.application_id().forget_abi();
                self.runtime.unsubscribe_from_events(origin_chain_id, app_id, BLACKJACK_STREAM_NAME.into());
                log::info!("\nUser {:?} unsubscribe from Play Chain {:?}\n", origin_chain_id, self.runtime.chain_id());
            }
            BlackjackMessage::RequestTableSeat { seat_id, balance } => {
                if self.request_table_seat_manager(seat_id, balance, origin_chain_id).is_some() {
                    let game = self.state.game.get();
                    self.channel_manager(BlackjackEvent::GameState { game: game.data_for_channel() })
                }
                log::info!("\nUser {:?} RequestTableSeat to Play Chain {:?}\n", origin_chain_id, self.runtime.chain_id());
            }
        }
    }

    // * Stream Subscriber
    async fn process_streams(&mut self, updates: Vec<StreamUpdate>) {
        for update in updates {
            assert_eq!(update.stream_id.stream_name, BLACKJACK_STREAM_NAME.into());
            assert_eq!(update.stream_id.application_id, self.runtime.application_id().forget_abi().into());
            for index in update.new_indices() {
                let event = self.runtime.read_event(update.chain_id, BLACKJACK_STREAM_NAME.into(), index);
                match event {
                    BlackjackEvent::GameState { game } => {
                        log::info!("\nUser {:?} received new game state:\n {:?}", self.runtime.chain_id(), game);
                        self.state.channel_game_state.set(game);
                    }
                }
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

    fn update_bankroll_balance(&mut self, amount: Amount) {
        let owner = self.runtime.application_id().into();
        let bankroll_app_id = self.runtime.application_parameters().bankroll;
        self.runtime
            .call_application(true, bankroll_app_id, &BankrollOperation::UpdateBalance { owner, amount });
    }

    // * User Chain
    fn create_single_player_blackjack_game(&mut self) -> BlackjackGame {
        let mut new_card_stack = get_new_deck(self.runtime.system_time().to_string());
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        BlackjackGame::new(Deck::with_cards(new_card_stack))
    }
    fn update_profile_balance_and_bet_data(&mut self) {
        let balance = self.get_bankroll_balance();
        let profile = self.state.profile.get_mut();
        profile.update_balance(balance);
        profile.calculate_bet_data();
    }
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
    fn process_find_play_chain_result(&mut self, origin_chain_id: ChainId, chain_id: Option<ChainId>) -> bool {
        if let Some(chain) = chain_id {
            log::info!("\nFindPlayChain Result Received at {:?} from: {:?}\n", self.runtime.chain_id(), origin_chain_id);
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
    fn add_user_to_new_single_player_game(&mut self) {
        let balance = self.state.profile.get().balance;
        let chain_id = self.runtime.chain_id();
        let seat_id: u8 = 1;
        let new_player = Player::new(seat_id, balance, chain_id);

        self.state.player_seat_map.insert(&seat_id, new_player.clone()).unwrap_or_else(|_| {
            panic!("Failed to update Player Seat Map for {:?} on add_user_to_new_single_player_game", chain_id);
        });
        self.state.user_status.set(UserStatus::InSinglePlayerGame);
        self.state.profile.get_mut().update_seat(seat_id);

        let mut blackjack_game = self.create_single_player_blackjack_game();
        blackjack_game.update_status(BlackjackStatus::WaitingForBets);
        blackjack_game.register_player(seat_id, new_player);
        self.state.single_player_game.set(blackjack_game);
    }
    fn add_user_to_new_multi_player_game(&mut self, seat_id: u8) {
        let balance = self.state.profile.get().balance;
        let chain_id = self.runtime.chain_id();
        self.state
            .player_seat_map
            .insert(&seat_id, Player::new(seat_id, balance, chain_id))
            .unwrap_or_else(|_| {
                panic!("Failed to update Player Seat Map for {:?} on add_user_to_new_multi_player_game", chain_id);
            });
        self.state.profile.get_mut().update_seat(seat_id);
        self.state.user_status.set(UserStatus::InMultiPlayerGame);
    }
    async fn player_bet(&mut self, amount: Amount) {
        if self.state.profile.get().bet_data.is_none() {
            panic!("missing Bet Data for placing bet");
        }

        let user_profile = self.state.profile.get().clone();
        let bet_data = user_profile.bet_data.unwrap();

        if user_profile.seat.is_none() {
            panic!("missing Player Seat ID");
        }
        if user_profile.balance.eq(&Amount::ZERO) || user_profile.balance.lt(&bet_data.min_bet) {
            panic!("not enough Player balance");
        }
        if amount.lt(&bet_data.min_bet) {
            panic!("minimum bet is {:?}", bet_data.min_bet);
        }
        if amount.gt(&bet_data.max_bet) {
            panic!("maximum bet is {:?}", bet_data.max_bet);
        }

        let seat_id = user_profile.seat.unwrap();
        let player_async = self.state.player_seat_map.get_mut(&seat_id).await;
        let player = player_async.expect("Player not found!").expect("Player not found!");

        player.add_bet(amount, user_profile.balance);
    }
    async fn deal_single_player(&mut self) {
        let profile = self.state.profile.get_mut();
        let seat_id = profile.seat;
        if seat_id.is_none() {
            panic!("missing Player Seat ID");
        }

        let bet_data = &profile.bet_data;
        if bet_data.is_none() {
            panic!("missing Bet Data");
        }

        let player_async = self.state.player_seat_map.get_mut(&seat_id.unwrap()).await;
        let player = player_async.expect("Player not found!").expect("Player not found!");

        let (bet_amount, latest_balance) = player.deal(bet_data.clone().unwrap().min_bet, profile.balance);
        profile.update_balance(latest_balance);
        self.update_bankroll_balance(latest_balance);

        let blackjack_token_pool = self.state.blackjack_token_pool.get_mut();
        blackjack_token_pool.saturating_add_assign(bet_amount);
    }
    async fn draw_single_player(&mut self) {
        let seat_id = self.state.profile.get().seat;
        if seat_id.is_none() {
            panic!("missing Player Seat ID");
        }

        let blackjack_game = self.state.single_player_game.get_mut();
        blackjack_game.draw_initial_cards(seat_id.unwrap());
        blackjack_game.update_status(BlackjackStatus::PlayerTurn);
    }
    // * Play Chain
    fn channel_manager(&mut self, event: BlackjackEvent) {
        self.runtime.emit(BLACKJACK_STREAM_NAME.into(), &event);
    }
    fn request_table_seat_manager(&mut self, seat_id: u8, balance: Amount, origin_chain_id: ChainId) -> Option<()> {
        let game = self.state.game.get_mut();

        if game.is_seat_taken(seat_id) {
            self.message_manager(origin_chain_id, BlackjackMessage::RequestTableSeatResult { seat_id, success: false });
            return None;
        }

        let player = Player::new(seat_id, balance, origin_chain_id);
        game.register_player(seat_id, player);
        self.message_manager(origin_chain_id, BlackjackMessage::RequestTableSeatResult { seat_id, success: true });
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
