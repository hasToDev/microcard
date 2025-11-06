use bankroll::{DailyBonus, DebtRecord};
use linera_sdk::linera_base_types::{AccountOwner, Amount};
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct BankrollState {
    // All Chain
    pub blackjack_token: RegisterView<Amount>,
    pub debt_log: MapView<u64, DebtRecord>,
    // User Chain
    pub daily_bonus: RegisterView<DailyBonus>,
    pub accounts: MapView<AccountOwner, Amount>,
}
