#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BankrollState;
use bankroll::{BankrollOperation, BankrollParameters, BankrollResponse};
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
    type Message = ();
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
            BankrollOperation::Balance { owner } => {
                log::info!("BankrollOperation::Balance request from  {:?}", owner);
                let mut balance = self
                    .state
                    .accounts
                    .get(&owner)
                    .await
                    .unwrap_or_else(|_| {
                        panic!("unable to get {:?} balance", owner);
                    })
                    .unwrap_or_default();

                let daily_bonus = self.state.daily_bonus.get_mut();
                if daily_bonus.is_zero() {
                    daily_bonus.update_bonus(self.runtime.application_parameters().bonus);
                }
                balance.saturating_add_assign(daily_bonus.claim_bonus(self.runtime.system_time()));

                self.state.accounts.insert(&owner, balance).unwrap_or_else(|_| {
                    panic!("unable to update {:?} balance", owner);
                });

                BankrollResponse::Balance(balance)
            }
        }
    }

    async fn execute_message(&mut self, _message: Self::Message) {}

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}
