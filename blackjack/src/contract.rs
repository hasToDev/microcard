#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BlackjackState;
use abi::blackjack::{BlackjackGame, BlackjackStatus, GameOutcome, MutationReason, UserStatus, BLACKJACK_STREAM_NAME, MAX_BLACKJACK_PLAYERS};
use abi::deck::{calculate_hand_value, format_card, get_new_deck, Deck};
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

const ONE_MINUTE_DURATION_IN_MICROS: u64 = 60 * 1_000_000;
const TWO_MINUTES_DURATION_IN_MICROS: u64 = 120 * 1_000_000;

const MINIMUM_BLACKJACK_DECK: u64 = 80;
const REFILL_BLACKJACK_DECK_COUNT: u64 = 364;

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
                log::info!("\n\nBlackjackOperation::SubscribeTo chain_id: {:?}", chain_id);
                self.message_manager(chain_id, BlackjackMessage::Subscribe);
                log::info!("Sent Subscribe message to chain_id: {:?}", chain_id);
            }
            BlackjackOperation::UnsubscribeFrom { chain_id } => {
                log::info!("\n\nBlackjackOperation::UnsubscribeFrom chain_id: {:?}", chain_id);
                self.message_manager(chain_id, BlackjackMessage::Unsubscribe);
                log::info!("Sent Unsubscribe message to chain_id: {:?}", chain_id);
            }
            BlackjackOperation::FindPlayChain {} => {
                log::info!("\n\nBlackjackOperation::FindPlayChain");

                match self.state.user_status.get() {
                    UserStatus::FindPlayChain => {
                        panic!("still waiting response from previous FindPlayChain");
                    }
                    UserStatus::InMultiPlayerGame | UserStatus::InSinglePlayerGame => {
                        panic!("user already in game, FindPlayChain not allowed");
                    }
                    UserStatus::RequestingTableSeat => {
                        panic!("user is requesting table seat, FindPlayChain not allowed");
                    }
                    UserStatus::PlayChainFound | UserStatus::RequestTableSeatFail => {
                        let play_chain_id = self.state.user_play_chain.get().unwrap();
                        self.message_manager(play_chain_id, BlackjackMessage::Unsubscribe);
                        self.state.user_play_chain.set(None);
                    }
                    _ => {}
                }

                let chain_id = self.get_public_chain();
                log::info!("Selected public chain: {:?} for FindPlayChain query", chain_id);
                self.state.user_status.set(UserStatus::FindPlayChain);
                self.state.find_play_chain_retry.set(0);
                self.message_manager(chain_id, BlackjackMessage::FindPlayChain);
                log::info!("Sent FindPlayChain message to public chain: {:?}", chain_id);
            }
            BlackjackOperation::RequestTableSeat { seat_id } => {
                log::info!("\n\nBlackjackOperation::RequestTableSeat for seat_id: {}", seat_id);
                if self.state.user_play_chain.get().is_none() {
                    panic!("no Play Chain found");
                }

                if seat_id == 0 || seat_id > MAX_BLACKJACK_PLAYERS as u8 {
                    panic!("seat_id is invalid, can only be 1-{:?}", MAX_BLACKJACK_PLAYERS);
                }

                match self.state.user_status.get() {
                    UserStatus::Idle | UserStatus::FindPlayChain | UserStatus::PlayChainUnavailable => {
                        panic!("please call FindPlayChain first");
                    }
                    UserStatus::RequestingTableSeat => {
                        panic!("still waiting response from previous RequestTableSeat");
                    }
                    UserStatus::InMultiPlayerGame | UserStatus::InSinglePlayerGame => {
                        panic!("user already in game, can't request new seat");
                    }
                    _ => {}
                }

                let balance = self.state.profile.get().balance;
                let play_chain_id = self.state.user_play_chain.get().unwrap();
                log::info!("Requesting seat_id: {} on play_chain: {:?} with balance: {}", seat_id, play_chain_id, balance);
                self.message_manager(play_chain_id, BlackjackMessage::RequestTableSeat { seat_id, balance });
                self.state.user_status.set(UserStatus::RequestingTableSeat);
                log::info!("Sent RequestTableSeat message to play_chain: {:?}", play_chain_id);
            }
            BlackjackOperation::GetBalance {} => {
                log::info!("\n\nBlackjackOperation::GetBalance");
                let balance = self.bankroll_get_balance();
                log::info!("Current Balance is {:?}", balance);
            }
            BlackjackOperation::Bet { amount } => {
                log::info!("\n\nBlackjackOperation::Bet amount: {}", amount);
                match self.state.user_status.get() {
                    UserStatus::InMultiPlayerGame => {
                        let game_status = &self.state.event_game_state.get().status;
                        match game_status {
                            BlackjackStatus::WaitingForPlayer | BlackjackStatus::PlayerTurn | BlackjackStatus::DealerTurn => {
                                panic!("game in play, not ready for placing bets, please wait for the next hands");
                            }
                            BlackjackStatus::RoundEnded => {
                                // TODO: prepare necessary thing before starting the next round if any
                            }
                            _ => {}
                        }

                        log::info!("Bet MultiPlayerGame, amount: {}", amount);
                        self.multi_player_player_bet(amount).await;
                    }
                    UserStatus::InSinglePlayerGame => {
                        let game_status = &self.state.single_player_game.get().status;
                        match game_status {
                            BlackjackStatus::WaitingForPlayer | BlackjackStatus::PlayerTurn | BlackjackStatus::DealerTurn => {
                                panic!("game in play, not ready for placing bets, please wait for the next hands");
                            }
                            BlackjackStatus::RoundEnded => {
                                self.update_profile_balance_and_bet_data();
                                self.prepare_next_single_player_bet_round().await;
                            }
                            _ => {}
                        }

                        log::info!("Bet SinglePlayerGame, amount: {}", amount);
                        self.player_bet(amount).await;
                    }
                    _ => {
                        panic!("Player not in any Single or MultiPlayerGame!");
                    }
                }
            }
            BlackjackOperation::DealBet {} => {
                log::info!("\n\nBlackjackOperation::DealBet");
                match self.state.user_status.get() {
                    UserStatus::InMultiPlayerGame => {
                        panic!("multi player deal bet not implemented yet");
                    }
                    UserStatus::InSinglePlayerGame => {
                        if self.state.single_player_game.get().status.ne(&BlackjackStatus::WaitingForBets) {
                            panic!("game in play, not ready for dealing bets, please wait for the next hands");
                        }
                        log::info!("DealBet SinglePlayerGame");
                        let outcome = self.deal_draw_single_player().await;
                        self.check_deck_single_player().await;

                        // Handle outcome based on initial deal
                        match outcome {
                            GameOutcome::PlayerWins => {
                                self.handle_player_win().await;
                            }
                            GameOutcome::DealerWins => {
                                self.handle_player_bust().await;
                            }
                            _ => {
                                // No Blackjack on initial deal, game continues normally
                                log::info!("Game continues to player turn");

                                // Update single_player_game state sequence
                                let current_time = self.runtime.system_time().micros();
                                let single_player_game = self.state.single_player_game.get_mut();
                                single_player_game.sequence = single_player_game.sequence.saturating_add(1);
                                single_player_game.set_time_limit(current_time, ONE_MINUTE_DURATION_IN_MICROS);
                            }
                        }
                    }
                    _ => {
                        panic!("Player not in any Single or MultiPlayerGame!");
                    }
                }
            }
            BlackjackOperation::Hit {} => {
                log::info!("\n\nBlackjackOperation::Hit");
                match self.state.user_status.get() {
                    UserStatus::InMultiPlayerGame => {
                        panic!("multi player hit not implemented yet");
                    }
                    UserStatus::InSinglePlayerGame => {
                        if self.state.single_player_game.get().status.ne(&BlackjackStatus::PlayerTurn) {
                            panic!("not the player turn");
                        }
                        log::info!("Hit SinglePlayerGame");
                        let outcome = self.hit_single_player().await;
                        self.check_deck_single_player().await;

                        // call handler based on outcome
                        match outcome {
                            GameOutcome::PlayerWins => {
                                self.handle_player_win().await;
                            }
                            GameOutcome::DealerWins => {
                                self.handle_player_bust().await;
                            }
                            GameOutcome::None => {
                                // Update single_player_game state sequence
                                let current_time = self.runtime.system_time().micros();
                                let single_player_game = self.state.single_player_game.get_mut();
                                single_player_game.sequence = single_player_game.sequence.saturating_add(1);
                                single_player_game.set_time_limit(current_time, ONE_MINUTE_DURATION_IN_MICROS);
                            }
                            _ => {
                                panic!("BlackjackOperation::Hit have unexpected outcome!");
                            }
                        }
                    }
                    _ => {
                        panic!("Player not in any Single or MultiPlayerGame!");
                    }
                }
            }
            BlackjackOperation::Stand {} => {
                log::info!("\n\nBlackjackOperation::Stand");
                match self.state.user_status.get() {
                    UserStatus::InMultiPlayerGame => {
                        panic!("multi player stand not implemented yet");
                    }
                    UserStatus::InSinglePlayerGame => {
                        if self.state.single_player_game.get().status.ne(&BlackjackStatus::PlayerTurn) {
                            panic!("not the player turn");
                        }
                        log::info!("Stand SinglePlayerGame");
                        let outcome = self.stand_single_player().await;
                        self.check_deck_single_player().await;

                        // call handler based on outcome
                        match outcome {
                            GameOutcome::PlayerWins => {
                                self.handle_player_win().await;
                            }
                            GameOutcome::DealerWins => {
                                self.handle_player_bust().await;
                            }
                            GameOutcome::Draw => {
                                self.handle_player_draw().await;
                            }
                            _ => {
                                panic!("BlackjackOperation::Stand have unexpected outcome!");
                            }
                        }
                    }
                    _ => {
                        panic!("Player not in any Single or MultiPlayerGame!");
                    }
                }
            }
            BlackjackOperation::StartSinglePlayerGame {} => {
                log::info!("\n\nBlackjackOperation::StartSinglePlayerGame");
                match self.state.user_status.get() {
                    UserStatus::Idle | UserStatus::PlayChainUnavailable => {
                        self.update_profile_balance_and_bet_data();
                        self.add_user_to_new_single_player_game();
                        let token_pool_address = self.get_public_chain();
                        log::info!("Token pool address set to: {:?}", token_pool_address);
                        self.state.token_pool_address.set(Some(token_pool_address));
                        log::info!("Single player game initialized successfully, user status: {:?}", self.state.user_status.get());
                    }
                    current_status => {
                        panic!("Unable to Start Single Player Game, user status is {:?}", current_status);
                    }
                }
            }
            BlackjackOperation::ExitSinglePlayerGame {} => {
                log::info!("\n\nBlackjackOperation::ExitSinglePlayerGame");
                let current_user_status = self.state.user_status.get();
                log::info!("Current user status: {:?}", current_user_status);
                if current_user_status.ne(&UserStatus::InSinglePlayerGame) {
                    panic!("Player not in any SinglePlayerGame!");
                }

                let game_status = &self.state.single_player_game.get().status;
                log::info!("Current game status: {:?}", game_status);
                if game_status.eq(&BlackjackStatus::WaitingForPlayer)
                    || game_status.eq(&BlackjackStatus::PlayerTurn)
                    || game_status.eq(&BlackjackStatus::DealerTurn)
                {
                    panic!("game in play, unable to exit, please finish the current game");
                }

                self.update_profile_balance_and_bet_data();
                self.state.user_status.set(UserStatus::Idle);
                self.state.single_player_game.clear();
                self.state.player_seat_map.clear();

                log::info!("Successfully exited single player game");
            }
            // * Master Chain
            BlackjackOperation::AddPlayChain {
                target_public_chain,
                play_chain_id,
            } => {
                log::info!("\n\nBlackjackOperation::AddPlayChain");
                assert_eq!(
                    self.runtime.chain_id(),
                    self.runtime.application_parameters().master_chain,
                    "MasterChain Authorization Required for BankrollOperation::AddPlayChain"
                );
                log::info!(
                    "BlackjackOperation::AddPlayChain at {:?}, target_public_chain: {:?}, play_chain_id: {:?}",
                    self.runtime.authenticated_signer(),
                    target_public_chain,
                    play_chain_id
                );
                self.message_manager(target_public_chain, BlackjackMessage::AddPlayChain { chain_id: play_chain_id });
                log::info!("Sent AddPlayChain message to target_public_chain: {:?}", target_public_chain);
            }
            BlackjackOperation::MintToken { chain_id, amount } => {
                log::info!("\n\nBlackjackOperation::MintToken");
                assert_eq!(
                    self.runtime.chain_id(),
                    self.runtime.application_parameters().master_chain,
                    "MasterChain Authorization Required for BlackjackOperation::MintToken"
                );
                log::info!(
                    "BlackjackOperation::MintToken at {:?}, minting {} tokens for chain: {:?}",
                    self.runtime.authenticated_signer(),
                    amount,
                    chain_id
                );
                let bankroll_app_id = self.runtime.application_parameters().bankroll;
                self.runtime
                    .call_application(true, bankroll_app_id, &BankrollOperation::MintToken { chain_id, amount });
                log::info!("Called bankroll MintToken for chain: {:?}, amount: {}", chain_id, amount);
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let origin_chain_id = self.runtime.message_origin_chain_id().expect("Chain ID missing from message");

        match message {
            // * User Chain
            BlackjackMessage::FindPlayChainResult { chain_id } => {
                log::info!("\n\nBlackjackMessage::FindPlayChainResult");
                log::info!("BlackjackMessage::FindPlayChainResult from {:?}, chain_id: {:?}", origin_chain_id, chain_id);
                if self.process_find_play_chain_result(origin_chain_id, chain_id) {
                    log::info!("Play chain found successfully, updating profile balance and bet data");
                    self.update_profile_balance_and_bet_data();
                }
            }
            BlackjackMessage::RequestTableSeatResult { seat_id, success } => {
                log::info!("\n\nBlackjackMessage::RequestTableSeatResult");
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
                log::info!("\n\nBlackjackMessage::FindPlayChain");
                log::info!("FindPlayChain Request Accepted at {:?} from: {:?}", self.runtime.chain_id(), origin_chain_id);

                let result = self.search_available_play_chain().await;
                self.message_manager(origin_chain_id, BlackjackMessage::FindPlayChainResult { chain_id: result });
            }
            BlackjackMessage::AddPlayChain { chain_id } => {
                log::info!("\n\nBlackjackMessage::AddPlayChain");
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
                log::info!("\n\nBlackjackMessage::Subscribe");
                let app_id = self.runtime.application_id().forget_abi();
                self.runtime.subscribe_to_events(origin_chain_id, app_id, BLACKJACK_STREAM_NAME.into());
                log::info!("User {:?} subscribe to Play Chain {:?}", origin_chain_id, self.runtime.chain_id());
            }
            BlackjackMessage::Unsubscribe => {
                log::info!("\n\nBlackjackMessage::Unsubscribe");
                let app_id = self.runtime.application_id().forget_abi();
                self.runtime.unsubscribe_from_events(origin_chain_id, app_id, BLACKJACK_STREAM_NAME.into());
                log::info!("User {:?} unsubscribe from Play Chain {:?}", origin_chain_id, self.runtime.chain_id());
            }
            BlackjackMessage::RequestTableSeat { seat_id, balance } => {
                log::info!("\n\nBlackjackMessage::RequestTableSeat");
                if self.request_table_seat_manager(seat_id, balance, origin_chain_id).is_some() {
                    let game = self.state.game.get();
                    self.event_manager(BlackjackEvent::GameState { game: game.data_for_event() })
                }
                log::info!("User {:?} RequestTableSeat to Play Chain {:?}", origin_chain_id, self.runtime.chain_id());
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
                        self.state.event_game_state.set(game);
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

    fn bankroll_get_balance(&mut self) -> Amount {
        let owner = self.runtime.application_id().into();
        let bankroll_app_id = self.runtime.application_parameters().bankroll;
        let response = self.runtime.call_application(true, bankroll_app_id, &BankrollOperation::Balance { owner });
        match response {
            BankrollResponse::Balance(balance) => balance,
            response => panic!("Unexpected response from Bankroll application: {response:?}"),
        }
    }

    fn bankroll_update_balance(&mut self, amount: Amount) {
        let owner = self.runtime.application_id().into();
        let bankroll_app_id = self.runtime.application_parameters().bankroll;
        self.runtime
            .call_application(true, bankroll_app_id, &BankrollOperation::UpdateBalance { owner, amount });
    }

    fn bankroll_notify_debt(&mut self, amount: Amount, target_chain: ChainId) {
        let bankroll_app_id = self.runtime.application_parameters().bankroll;
        self.runtime
            .call_application(true, bankroll_app_id, &BankrollOperation::NotifyDebt { amount, target_chain });
    }

    fn bankroll_transfer_token_pot(&mut self, amount: Amount, target_chain: ChainId) {
        let bankroll_app_id = self.runtime.application_parameters().bankroll;
        self.runtime
            .call_application(true, bankroll_app_id, &BankrollOperation::TransferTokenPot { amount, target_chain });
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
    fn refill_deck(&mut self) -> Vec<u8> {
        let mut new_card_stack = get_new_deck(self.runtime.system_time().to_string());
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack.append(&mut get_new_deck(self.runtime.system_time().to_string()));
        new_card_stack
    }
    async fn check_deck_single_player(&mut self) {
        let deck_count = self.state.single_player_game.get().count;

        // Ensure deck has enough cards
        if deck_count < MINIMUM_BLACKJACK_DECK {
            let mut refill_deck = self.refill_deck();
            let current_time = self.runtime.system_time().to_string();
            let single_player_game = self.state.single_player_game.get_mut();
            single_player_game.deck.add_cards(&mut refill_deck, current_time);
            single_player_game.count = single_player_game.count.saturating_add(REFILL_BLACKJACK_DECK_COUNT);
        }
    }
    fn update_profile_balance_and_bet_data(&mut self) {
        log::info!("Updating profile balance and bet data");
        let balance = self.bankroll_get_balance();
        log::info!("Retrieved balance from bankroll: {}", balance);
        let profile = self.state.profile.get_mut();
        profile.update_balance(balance);
        profile.calculate_bet_data();
        log::info!("Profile updated - balance: {}, bet_data: {:?}", balance, profile.bet_data);
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
        let seat_id: u8 = 0; // always 0 for single player
        log::info!(
            "Adding user to new single player game - chain_id: {:?}, balance: {}, seat_id: {}",
            chain_id,
            balance,
            seat_id
        );
        let new_player = Player::new(seat_id, balance, chain_id);

        self.state.player_seat_map.insert(&seat_id, new_player.clone()).unwrap_or_else(|_| {
            panic!("Failed to update Player Seat Map for {:?} on add_user_to_new_single_player_game", chain_id);
        });
        self.state.user_status.set(UserStatus::InSinglePlayerGame);
        self.state.profile.get_mut().update_seat(seat_id);

        let current_time = self.runtime.system_time().micros();
        let mut single_player_game = self.create_single_player_blackjack_game();
        single_player_game.update_status(BlackjackStatus::WaitingForBets);
        single_player_game.register_update_player(seat_id, new_player);
        single_player_game.sequence = single_player_game.sequence.saturating_add(1);
        single_player_game.set_time_limit(current_time, ONE_MINUTE_DURATION_IN_MICROS);
        self.state.single_player_game.set(single_player_game.clone());
        log::info!("Single player game created successfully - status: {:?}", single_player_game.status);
    }
    fn add_user_to_new_multi_player_game(&mut self, seat_id: u8) {
        let balance = self.state.profile.get().balance;
        let chain_id = self.runtime.chain_id();
        log::info!(
            "Adding user to multi player game - chain_id: {:?}, balance: {}, seat_id: {}",
            chain_id,
            balance,
            seat_id
        );
        self.state
            .player_seat_map
            .insert(&seat_id, Player::new(seat_id, balance, chain_id))
            .unwrap_or_else(|_| {
                panic!("Failed to update Player Seat Map for {:?} on add_user_to_new_multi_player_game", chain_id);
            });
        self.state.profile.get_mut().update_seat(seat_id);
        self.state.user_status.set(UserStatus::InMultiPlayerGame);
        log::info!("User successfully joined multi player game at seat: {}", seat_id);
    }
    async fn player_bet(&mut self, amount: Amount) {
        log::info!("player_bet called with amount: {}", amount);
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
        if amount.lt(&Amount::ZERO) {
            panic!("negative bet isn't allowed");
        }
        if amount.gt(&Amount::ZERO) && amount.lt(&bet_data.min_bet) {
            panic!("minimum bet is {:?}", bet_data.min_bet);
        }
        if amount.gt(&bet_data.max_bet) {
            panic!("maximum bet is {:?}", bet_data.max_bet);
        }

        let seat_id = user_profile.seat.unwrap();
        log::info!(
            "Bet validation passed - seat_id: {}, amount: {}, min_bet: {}, max_bet: {}, balance: {}",
            seat_id,
            amount,
            bet_data.min_bet,
            bet_data.max_bet,
            user_profile.balance
        );

        // Retrieve player and add bet
        let single_player_game = self.state.single_player_game.get_mut();
        single_player_game.sequence = single_player_game.sequence.saturating_add(1);

        let player = single_player_game.players.get_mut(&seat_id).expect("Player not found in single player game");
        player.update_bet(amount, user_profile.balance);

        // Update player in player_seat_map
        self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
            panic!("Failed to update Player Seat Map on player_bet");
        });

        log::info!("Bet placed successfully for seat_id: {}, amount: {}", seat_id, amount);
    }

    async fn multi_player_player_bet(&mut self, amount: Amount) {
        // TODO: continue
    }

    async fn deal_draw_single_player(&mut self) -> GameOutcome {
        log::info!("deal_draw_single_player called");
        let profile = self.state.profile.get_mut();
        let seat_id = profile.seat.expect("missing Seat ID");

        let bet_data = &profile.bet_data;
        if bet_data.is_none() {
            panic!("missing Bet Data");
        }

        let single_player_game = self.state.single_player_game.get_mut();
        log::info!("Initial cards drawn for seat_id: {}", seat_id);
        single_player_game.draw_initial_cards(seat_id);
        single_player_game.update_status(BlackjackStatus::PlayerTurn);

        let player = single_player_game.players.get_mut(&seat_id).expect("Player not found");
        player.current_player = true;

        let (bet_amount, latest_balance) = player.deal_bet(bet_data.clone().unwrap().min_bet, profile.balance);
        log::info!("Player dealt - bet_amount: {}, latest_balance: {}", bet_amount, latest_balance);
        profile.update_balance(latest_balance);
        single_player_game.pot.saturating_add_assign(bet_amount);
        log::info!("Deal complete - game pot: {}, player balance: {}", single_player_game.pot, latest_balance);

        let blackjack_token_pool = self.state.blackjack_token_pool.get_mut();
        let previous_pool = *blackjack_token_pool;
        blackjack_token_pool.saturating_add_assign(bet_amount);
        log::info!("Token pool updated: {} -> {}", previous_pool, blackjack_token_pool);

        // Calculate hand values for both dealer and player
        let dealer_hand_value = calculate_hand_value(&single_player_game.dealer.hand);
        let player_hand_value = calculate_hand_value(&player.hand);

        log::info!(
            "Initial deal - Dealer hand value: {}, Player hand value: {}",
            dealer_hand_value,
            player_hand_value
        );

        // Update player in player_seat_map
        self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
            panic!("Failed to update Player Seat Map on deal_draw_single_player");
        });

        // Check for Blackjack (21) in initial deal
        let outcome = match (dealer_hand_value == 21, player_hand_value == 21) {
            (true, true) => {
                log::info!("Both dealer and player have Blackjack! It's a draw");
                GameOutcome::Draw
            }
            (true, false) => {
                log::info!("Dealer has Blackjack! Dealer wins");
                GameOutcome::DealerWins
            }
            (false, true) => {
                log::info!("Player has Blackjack! Player wins");
                GameOutcome::PlayerWins
            }
            (false, false) => {
                log::info!("No Blackjack on initial deal, game continues");
                GameOutcome::None
            }
        };

        self.bankroll_update_balance(latest_balance);

        outcome
    }
    // * Play Chain
    fn event_manager(&mut self, event: BlackjackEvent) {
        self.runtime.emit(BLACKJACK_STREAM_NAME.into(), &event);
    }
    fn request_table_seat_manager(&mut self, seat_id: u8, balance: Amount, origin_chain_id: ChainId) -> Option<()> {
        log::info!(
            "request_table_seat_manager - seat_id: {}, balance: {}, origin_chain: {:?}",
            seat_id,
            balance,
            origin_chain_id
        );
        let game = self.state.game.get_mut();

        if game.is_seat_taken(seat_id) {
            log::info!("Seat {} is already taken, rejecting request from {:?}", seat_id, origin_chain_id);
            self.message_manager(origin_chain_id, BlackjackMessage::RequestTableSeatResult { seat_id, success: false });
            return None;
        }

        log::info!("Seat {} is available, registering player from {:?}", seat_id, origin_chain_id);
        let player = Player::new(seat_id, balance, origin_chain_id);
        game.register_update_player(seat_id, player);
        self.message_manager(origin_chain_id, BlackjackMessage::RequestTableSeatResult { seat_id, success: true });
        log::info!("Player from {:?} successfully registered at seat {}", origin_chain_id, seat_id);
        Some(())
    }
    // * Public Chain
    async fn search_available_play_chain(&mut self) -> Option<ChainId> {
        for player_number in (0..MAX_BLACKJACK_PLAYERS).rev() {
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
        // REMOVAL PHASE: Remove from old bucket (for Update) or entirely (for Remove)
        if status == MutationReason::Update || status == MutationReason::Remove {
            // remove chain_id from the current play_chain_set state
            if let Some(old_state) = self.state.play_chain_status.get(&chain_id).await.unwrap_or_default() {
                let mut vec_data = self.state.play_chain_set.get(&old_state).await.unwrap_or_default().unwrap_or_default();

                // Optimized removal: find position and remove single element
                // More efficient than retain() which iterates entire vector to filter
                if let Some(pos) = vec_data.iter().position(|c| c == &chain_id) {
                    vec_data.remove(pos);
                }

                self.state.play_chain_set.insert(&old_state, vec_data).unwrap_or_else(|_| {
                    panic!("Failed to update Play Chain Set for {:?}", chain_id);
                });
            }
        }

        // ADDITION PHASE: Add to new bucket (skip for Remove)
        if status != MutationReason::Remove {
            // add chain_id to the new play_chain_set state
            let mut vec_data = self.state.play_chain_set.get(&player_number).await.unwrap_or_default().unwrap_or_default();

            // Defensive check: prevent duplicate entries
            if !vec_data.contains(&chain_id) {
                vec_data.push(chain_id);
            }

            self.state.play_chain_set.insert(&player_number, vec_data).unwrap_or_else(|_| {
                panic!("Failed to update Play Chain Set for {:?}", chain_id);
            });

            // update chain_id status on the play_chain_status
            self.state.play_chain_status.insert(&chain_id, player_number).unwrap_or_else(|_| {
                panic!("Failed to update Play Chain Status for {:?}", chain_id);
            });
        } else {
            // Remove case: delete chain from status tracking entirely
            self.state.play_chain_status.remove(&chain_id).unwrap_or_else(|_| {
                panic!("Failed to remove Play Chain Status for {:?}", chain_id);
            });
        }
    }

    // Hit operation: deal one card to player and calculate hand value
    async fn hit_single_player(&mut self) -> GameOutcome {
        // Retrieve seat in profile state
        let profile = self.state.profile.get();
        let seat_id = profile.seat.expect("Player seat not found");

        // Retrieve single_player_game state
        let single_player_game = self.state.single_player_game.get_mut();

        // Retrieve Player's object from single_player_game players based on the seat
        let player = single_player_game.players.get_mut(&seat_id).expect("Player not found in single player game");

        // Deal one card from deck and insert it into Player's object hand
        let card = single_player_game.deck.deal_card().expect("Deck ran out of cards");
        single_player_game.count = single_player_game.count.saturating_sub(1);
        player.hand.push(card);

        // Update player in player_seat_map
        self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
            panic!("Failed to update Player Seat Map on hit_single_player");
        });

        // Calculate the value in Player's object hand
        let hand_value = calculate_hand_value(&player.hand);

        log::info!("Player hit: drew {}, hand value is now {}", format_card(card), hand_value);

        let outcome = if hand_value > 21 {
            // Player busts
            log::info!("Player busts with {}", hand_value);
            GameOutcome::DealerWins
        } else if hand_value == 21 {
            // Player win
            log::info!("Player wins with {}", hand_value);
            GameOutcome::PlayerWins
        } else {
            // No outcome, player can keep dealing or choose to stand
            log::info!("Player hand value is {}", hand_value);
            GameOutcome::None
        };

        outcome
    }

    // Handle player win (hand value = 21)
    async fn handle_player_win(&mut self) {
        log::info!("Player wins!");

        let profile = self.state.profile.get();
        let seat_id = profile.seat.expect("Player seat not found");

        // Get player and calculate winnings (2:1 payout)
        let current_time = self.runtime.system_time().micros();
        let single_player_game = self.state.single_player_game.get_mut();
        single_player_game.update_status(BlackjackStatus::RoundEnded);
        single_player_game.sequence = single_player_game.sequence.saturating_add(1);
        single_player_game.set_time_limit(current_time, TWO_MINUTES_DURATION_IN_MICROS);
        let player = single_player_game.players.get_mut(&seat_id).expect("Player not found in single player game");

        // Player gets back bet + equal winnings
        let bet_amount = player.bet;
        let winnings = bet_amount.saturating_mul(2);

        log::info!("Player bet: {}, winnings: {}", bet_amount, winnings);

        // Check blackjack_token_pool availability
        let blackjack_token_pool = self.state.blackjack_token_pool.get();

        if *blackjack_token_pool >= winnings {
            // Sufficient funds in pool - pay normally
            log::info!("Sufficient pool funds. Paying {} from pool of {}", winnings, blackjack_token_pool);

            // Subtract winning amount from pool
            let remaining_token = blackjack_token_pool.saturating_sub(winnings);
            self.state.blackjack_token_pool.set(remaining_token);

            // Update player balance
            let new_balance = profile.balance.saturating_add(winnings);
            self.state.profile.get_mut().update_balance(new_balance);
            player.balance = new_balance;

            self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
                panic!("Failed to update Player Seat Map on handle_player_win");
            });

            self.bankroll_update_balance(new_balance);

            log::info!("Player paid. New balance: {}, Remaining pool: {}", new_balance, remaining_token);
        } else {
            // Insufficient funds - pay what's available and create debt
            let available = *blackjack_token_pool;
            let debt_amount = winnings.saturating_sub(available);

            log::info!("Insufficient funds! Available: {}, Required: {}, Debt: {}", available, winnings, debt_amount);

            // Pay player the winning amount
            let new_balance = profile.balance.saturating_add(winnings);
            self.state.profile.get_mut().update_balance(new_balance);
            player.balance = new_balance;

            self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
                panic!("Failed to update Player Seat Map on handle_player_win");
            });

            self.bankroll_update_balance(new_balance);

            // Create debt by calling Bankroll
            let token_pool_address = self.state.token_pool_address.get().expect("Token pool address not set");
            self.bankroll_notify_debt(debt_amount, token_pool_address);

            // Ensure pool is empty
            self.state.blackjack_token_pool.set(Amount::ZERO);
        }

        log::info!("Player win processed successfully");
    }

    // Handle player bust (hand value > 21)
    async fn handle_player_bust(&mut self) {
        log::info!("Player busts!");

        // Player loses - transfer entire pot to public chain
        let pot_amount = *self.state.blackjack_token_pool.get();

        if pot_amount > Amount::ZERO {
            log::info!("Transferring pot to public chain. Amount: {}", pot_amount);

            // Call Bankroll to transfer pot
            let token_pool_address = self.state.token_pool_address.get().expect("Token pool address not set");
            self.bankroll_transfer_token_pot(pot_amount, token_pool_address);

            // Reset pool to zero
            self.state.blackjack_token_pool.set(Amount::ZERO);

            log::info!("Token pot transferred. Amount: {}, Target: {:?}", pot_amount, token_pool_address);
        } else {
            log::info!("No tokens in pot to transfer");
        }

        // Update game status
        let current_time = self.runtime.system_time().micros();
        let game = self.state.single_player_game.get_mut();
        game.update_status(BlackjackStatus::RoundEnded);
        game.sequence = game.sequence.saturating_add(1);
        game.set_time_limit(current_time, TWO_MINUTES_DURATION_IN_MICROS);

        log::info!("Player bust processed successfully");
    }

    // Handle player draw (same hand value as dealer)
    async fn handle_player_draw(&mut self) {
        log::info!("It's a draw!");

        let profile = self.state.profile.get();
        let seat_id = profile.seat.expect("Player seat not found");

        // Get player's bet amount
        let current_time = self.runtime.system_time().micros();
        let single_player_game = self.state.single_player_game.get_mut();
        single_player_game.update_status(BlackjackStatus::RoundEnded);
        single_player_game.sequence = single_player_game.sequence.saturating_add(1);
        single_player_game.set_time_limit(current_time, TWO_MINUTES_DURATION_IN_MICROS);
        let player = single_player_game.players.get_mut(&seat_id).expect("Player not found in single player game");

        // Player gets back their bet (no winnings, no loss)
        let bet_amount = player.bet;

        log::info!("Player bet: {}, returning bet amount", bet_amount);

        // Check blackjack_token_pool availability
        let blackjack_token_pool = self.state.blackjack_token_pool.get();

        if *blackjack_token_pool >= bet_amount {
            // Sufficient funds in pool - return bet
            log::info!("Sufficient pool funds. Returning {} from pool of {}", bet_amount, blackjack_token_pool);

            // Subtract bet amount from pool
            let remaining_token = blackjack_token_pool.saturating_sub(bet_amount);
            self.state.blackjack_token_pool.set(remaining_token);

            // Update player balance (return bet)
            let new_balance = profile.balance.saturating_add(bet_amount);
            self.state.profile.get_mut().update_balance(new_balance);
            player.balance = new_balance;

            self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
                panic!("Failed to update Player Seat Map on handle_player_draw");
            });

            self.bankroll_update_balance(new_balance);

            log::info!("Bet returned. New balance: {}, Remaining pool: {}", new_balance, remaining_token);
        } else {
            // Insufficient funds - return what's available and create debt
            let available = *blackjack_token_pool;
            let debt_amount = bet_amount.saturating_sub(available);

            log::info!("Insufficient funds! Available: {}, Required: {}, Debt: {}", available, bet_amount, debt_amount);

            // Return bet to player
            let new_balance = profile.balance.saturating_add(bet_amount);
            self.state.profile.get_mut().update_balance(new_balance);
            player.balance = new_balance;

            self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
                panic!("Failed to update Player Seat Map on handle_player_draw");
            });

            self.bankroll_update_balance(new_balance);

            // Create debt by calling Bankroll
            let token_pool_address = self.state.token_pool_address.get().expect("Token pool address not set");
            self.bankroll_notify_debt(debt_amount, token_pool_address);

            // Ensure pool is empty
            self.state.blackjack_token_pool.set(Amount::ZERO);
        }

        log::info!("Draw processed successfully");
    }

    // Stand operation: dealer draws cards and compare hands
    async fn stand_single_player(&mut self) -> GameOutcome {
        log::info!("Processing stand operation");

        // Retrieve single_player_game state
        let single_player_game = self.state.single_player_game.get_mut();

        // Update status to DealerTurn
        single_player_game.update_status(BlackjackStatus::DealerTurn);

        // Calculate dealer hand value
        let mut dealer_hand_value = calculate_hand_value(&single_player_game.dealer.hand);
        log::info!("Initial dealer hand value: {}", dealer_hand_value);

        // Keep dealing cards to dealer if hand value is lower than 17
        while dealer_hand_value < 17 {
            let card = single_player_game.deck.deal_card().expect("Deck ran out of cards");
            single_player_game.dealer.hand.push(card);
            single_player_game.count = single_player_game.count.saturating_sub(1);
            dealer_hand_value = calculate_hand_value(&single_player_game.dealer.hand);
            log::info!("Dealer drew {}, hand value is now {}", format_card(card), dealer_hand_value);
        }

        log::info!("Dealer finished drawing. Final hand value: {}", dealer_hand_value);

        let profile = self.state.profile.get();
        let seat_id = profile.seat.expect("Player seat not found");
        let player = single_player_game.players.get(&seat_id).expect("Player not found in single player game");

        // Calculate the Player's hand value
        let player_hand_value = calculate_hand_value(&player.hand);
        log::info!("Player hand value: {}", player_hand_value);

        // Compare both Player and Dealer hand value based on Blackjack rules
        let outcome = if dealer_hand_value > 21 {
            // Dealer busts, player wins
            log::info!("Dealer busts with {}", dealer_hand_value);
            GameOutcome::PlayerWins
        } else if player_hand_value > dealer_hand_value {
            // Player has higher hand value
            log::info!("Player wins: {} vs {}", player_hand_value, dealer_hand_value);
            GameOutcome::PlayerWins
        } else if dealer_hand_value > player_hand_value {
            // Dealer has higher hand value
            log::info!("Dealer wins: {} vs {}", dealer_hand_value, player_hand_value);
            GameOutcome::DealerWins
        } else {
            // Same hand value
            log::info!("Draw: both have {}", player_hand_value);
            GameOutcome::Draw
        };

        log::info!("Stand operation completed. Outcome: {:?}", outcome);

        outcome
    }

    async fn prepare_next_single_player_bet_round(&mut self) {
        log::info!("Preparing for next single player bet round");

        let profile = self.state.profile.get();
        let seat_id = profile.seat.expect("Player seat not found");

        let single_player_game = self.state.single_player_game.get_mut();
        single_player_game.dealer.hand = vec![];
        single_player_game.pot = Amount::ZERO;
        single_player_game.update_status(BlackjackStatus::WaitingForBets);
        log::info!("Dealer hand and pot reset, Game status updated to WaitingForBets");

        let player = single_player_game.players.get_mut(&seat_id).expect("Player not found in single player game");

        // Update player balance from profile
        player.balance = profile.balance;
        log::info!("Player balance updated to: {}", player.balance);

        player.reset_bet();
        player.hand = vec![];
        log::info!("Player hand and bet resets");

        // Update player in player_seat_map
        self.state.player_seat_map.insert(&seat_id, player.clone()).unwrap_or_else(|_| {
            panic!("Failed to update Player Seat Map on prepare_for_next_single_player_bet_round");
        });

        log::info!("Preparation for next round complete");
    }
}
