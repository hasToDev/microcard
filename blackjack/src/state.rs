use abi::deck::Deck;
use linera_sdk::views::{linera_views, RegisterView, RootView, ViewStorageContext};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct BlackjackState {
    pub value: RegisterView<u64>,
    pub deck_card: RegisterView<Deck>,
    // Add fields here.
}
