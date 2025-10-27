use bankroll::DailyBonus;
use linera_sdk::linera_base_types::{AccountOwner, Amount};
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct BankrollState {
    pub token: RegisterView<Amount>,
    pub daily_bonus: RegisterView<DailyBonus>,
    pub accounts: MapView<AccountOwner, Amount>,
}
