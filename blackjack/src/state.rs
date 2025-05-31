use abi::blackjack::{BlackjackGame, UserStatus};
use abi::deck::Deck;
use abi::player_dealer::Player;
use linera_sdk::linera_base_types::ChainId;
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct BlackjackState {
    pub instantiate_value: RegisterView<u64>,
    // All Chain
    pub blackjack_token: RegisterView<u64>,
    // Public Chain
    pub play_chain_set: MapView<u8, Vec<ChainId>>,
    pub play_chain_status: MapView<ChainId, u8>,
    // User Chain
    pub player: MapView<u8, Player>,
    pub player_seat: RegisterView<u8>,
    pub user_status: RegisterView<UserStatus>,
    pub user_play_chain: RegisterView<Vec<ChainId>>,
    pub find_play_chain_retry: RegisterView<u8>,
    // Play Chain
    pub deck_card: RegisterView<Deck>,
    pub game: RegisterView<BlackjackGame>,
}
