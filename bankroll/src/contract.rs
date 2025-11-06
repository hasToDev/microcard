#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BankrollState;
use bankroll::{BankrollMessage, BankrollOperation, BankrollParameters, BankrollResponse, DebtRecord, DebtStatus, TokenPotRecord};
use linera_sdk::linera_base_types::ChainId;
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

pub struct BankrollContract {
    state: BankrollState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(BankrollContract);

impl WithContractAbi for BankrollContract {
    type Abi = bankroll::BankrollAbi;
}

impl Contract for BankrollContract {
    type Message = BankrollMessage;
    type Parameters = BankrollParameters;
    type InstantiationArgument = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BankrollState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        BankrollContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        // validate that the application parameters were configured correctly.
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            // * User Chain
            BankrollOperation::Balance { owner } => {
                log::info!("BankrollOperation::Balance request from  {:?}", owner);

                let balance_async = self.state.accounts.get(&owner).await;
                let mut balance = balance_async.expect("unable to get balance").unwrap_or_default();

                let daily_bonus = self.state.daily_bonus.get_mut();
                if daily_bonus.is_zero() {
                    daily_bonus.update_bonus(self.runtime.application_parameters().bonus);
                }
                balance.saturating_add_assign(daily_bonus.claim_bonus(self.runtime.system_time()));

                self.state.accounts.insert(&owner, balance).unwrap_or_else(|_| {
                    panic!("unable to update {:?} balance", owner);
                });

                log::info!("BankrollOperation::Balance returning balance: {} for owner: {:?}", balance, owner);
                BankrollResponse::Balance(balance)
            }
            BankrollOperation::UpdateBalance { owner, amount } => {
                log::info!("BankrollOperation::UpdateBalance request from {:?}, updating balance to: {}", owner, amount);

                self.state.accounts.insert(&owner, amount).unwrap_or_else(|_| {
                    panic!("unable to update {:?} balance", owner);
                });

                log::info!("BankrollOperation::UpdateBalance completed for owner: {:?}, new balance: {}", owner, amount);
                BankrollResponse::Ok
            }
            BankrollOperation::NotifyDebt { amount, target_chain } => {
                log::info!("BankrollOperation::NotifyDebt request from {:?}, amount: {}, target_chain: {:?}",
                    self.runtime.authenticated_signer(), amount, target_chain);

                let user_chain = self.runtime.chain_id();
                let created_at = self.runtime.system_time();
                let debt_id = created_at.micros();

                // Create debt record before sending notification
                let debt_record = DebtRecord {
                    id: debt_id,
                    user_chain,
                    amount,
                    created_at,
                    paid_at: None,
                    status: DebtStatus::Pending,
                };

                self.state.debt_log.insert(&debt_id, debt_record.clone()).unwrap_or_else(|_| {
                    panic!("Failed to create debt record for debt_id: {}", debt_id);
                });

                log::info!("Created debt record: {:?}", debt_record);

                self.message_manager(target_chain, BankrollMessage::DebtNotif { debt_id, amount, created_at });
                log::info!("Sent DebtNotif message to target_chain: {:?}, debt_id: {}", target_chain, debt_id);
                BankrollResponse::Ok
            }
            BankrollOperation::TransferTokenPot { amount, target_chain } => {
                log::info!("BankrollOperation::TransferTokenPot request from {:?}, amount: {}, target_chain: {:?}",
                    self.runtime.authenticated_signer(), amount, target_chain);

                self.message_manager(target_chain, BankrollMessage::TokenPot { amount });
                log::info!("Sent TokenPot message to target_chain: {:?}, amount: {}", target_chain, amount);
                BankrollResponse::Ok
            }
            // * Master Chain
            BankrollOperation::MintToken { chain_id, amount } => {
                assert_eq!(
                    self.runtime.chain_id(),
                    self.runtime.application_parameters().master_chain,
                    "MasterChain Authorization Required for BankrollOperation::MintToken"
                );
                log::info!("BankrollOperation::MintToken request from {:?}, minting {} tokens for chain: {:?}",
                    self.runtime.authenticated_signer(), amount, chain_id);
                self.message_manager(chain_id, BankrollMessage::ReceivedToken { amount });
                log::info!("Sent ReceivedToken message to chain: {:?}, amount: {}", chain_id, amount);
                BankrollResponse::Ok
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let origin_chain_id = self.runtime.message_origin_chain_id().expect("Chain ID missing from message");

        match message {
            // * Public Chain
            BankrollMessage::ReceivedToken { amount } => {
                log::info!("BankrollMessage::ReceivedToken from {:?} at {:?}, amount: {}", origin_chain_id, self.runtime.chain_id(), amount);
                let current_token = self.state.blackjack_token.get_mut();
                let previous_balance = *current_token;
                current_token.saturating_add_assign(amount);
                log::info!("Token balance updated: {} -> {}", previous_balance, current_token);
            }
            BankrollMessage::DebtNotif { debt_id, amount, created_at } => {
                log::info!(
                    "BankrollMessage::DebtNotif debt_id: {} from user_chain: {:?} amount: {} at {:?}",
                    debt_id,
                    origin_chain_id,
                    amount,
                    self.runtime.chain_id()
                );

                // Verify we have sufficient tokens
                let current_token = self.state.blackjack_token.get();
                log::info!("Current token pool before debt payment: {}", current_token);
                assert!(
                    *current_token >= amount,
                    "Insufficient tokens to pay debt. Available: {}, Required: {}",
                    current_token,
                    amount
                );

                // Subtract the debt amount from blackjack_token pool
                let remaining_token = current_token.saturating_sub(amount);
                self.state.blackjack_token.set(remaining_token);

                log::info!(
                    "Debt payment processed. Token pool: {} -> {}. Sending DebtPaid to {:?}",
                    current_token,
                    remaining_token,
                    origin_chain_id
                );

                // Send DebtPaid message back to the user chain
                let paid_at = self.runtime.system_time();
                self.message_manager(origin_chain_id, BankrollMessage::DebtPaid { debt_id, amount, paid_at });

                // Log debt history
                let debt_record = DebtRecord {
                    id: debt_id,
                    user_chain: origin_chain_id,
                    amount,
                    created_at,
                    paid_at: Some(paid_at),
                    status: DebtStatus::Paid,
                };
                self.state.debt_log.insert(&debt_id, debt_record.clone()).unwrap_or_else(|_| {
                    panic!("Failed to create debt record for debt_id: {}", debt_id);
                });
            }
            BankrollMessage::TokenPot { amount } => {
                log::info!(
                    "BankrollMessage::TokenPot from {:?} amount: {} at {:?}",
                    origin_chain_id,
                    amount,
                    self.runtime.chain_id()
                );

                // Add the pot amount to blackjack_token pool
                let current_token = self.state.blackjack_token.get_mut();
                current_token.saturating_add_assign(amount);

                // Create token pot record for history
                let created_at = self.runtime.system_time();
                let pot_id = created_at.micros();
                let pot_record = TokenPotRecord {
                    id: pot_id,
                    user_chain: origin_chain_id,
                    amount,
                    created_at,
                };

                self.state.token_pot_log.insert(&pot_id, pot_record.clone()).unwrap_or_else(|_| {
                    panic!("Failed to create token pot record for pot_id: {}", pot_id);
                });

                log::info!("Token pot received. New total tokens: {}. Pot record created: {:?}", current_token, pot_record);
            }
            // * User Chain
            BankrollMessage::DebtPaid { debt_id, amount, paid_at } => {
                log::info!(
                    "BankrollMessage::DebtPaid debt_id: {} amount: {} timestamp: {:?} at {:?}",
                    debt_id,
                    amount,
                    paid_at,
                    self.runtime.chain_id()
                );

                // Update the debt record with paid_at and status
                let mut debt_record = self
                    .state
                    .debt_log
                    .get(&debt_id)
                    .await
                    .expect("Failed to get debt record")
                    .expect("Debt record not found");

                debt_record.paid_at = Some(paid_at);
                debt_record.status = DebtStatus::Paid;

                self.state.debt_log.insert(&debt_id, debt_record).unwrap_or_else(|_| {
                    panic!("Failed to update debt record for debt_id: {}", debt_id);
                });

                log::info!("Debt {} successfully updated to Paid status", debt_id);
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl BankrollContract {
    fn message_manager(&mut self, destination: ChainId, message: BankrollMessage) {
        self.runtime.prepare_message(message).with_tracking().send_to(destination);
    }
}
