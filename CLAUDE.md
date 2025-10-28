# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a decentralized blackjack gaming application built on the Linera blockchain platform. The project implements a
multi-chain architecture where game logic, token management, and player state are distributed across different chain
types.

## Build and Development Commands

### Building the Project

```bash
# Build all workspace members for WebAssembly target
cargo build --release --target wasm32-unknown-unknown

# Build a specific package
cargo build -p blackjack --release --target wasm32-unknown-unknown
cargo build -p bankroll --release --target wasm32-unknown-unknown
cargo build -p abi --release --target wasm32-unknown-unknown

# Check code without building
cargo check

# Run clippy for linting
cargo clippy --target wasm32-unknown-unknown

# Format code
cargo fmt
```

### Testing

```bash
# Run tests (tests are currently commented out in tests/ directory)
cargo test
```

### Custom Commands

```bash
# Transfer script (uses WSL)
make transfer
```

## Architecture

### Workspace Structure

The project is organized as a Cargo workspace with three main crates:

- **`abi/`**: Shared data structures, types, and game logic used by both applications
    - Blackjack game state and rules (`blackjack.rs`)
    - Deck and card handling (`deck.rs`)
    - Player and dealer logic (`player_dealer.rs`)
    - Betting and chip profiles (`bet_chip_profile.rs`)
    - Random number generation (`random.rs`)
    - Poker-related utilities (`poker.rs`)

- **`bankroll/`**: Token and balance management application
    - Handles user balances and daily bonuses
    - Mints tokens on master chain
    - Provides balance queries via GraphQL

- **`blackjack/`**: Main blackjack game application
    - Implements both single-player and multi-player game modes
    - Manages table seats and game state
    - Integrates with bankroll for balance management

### Linera Multi-Chain Architecture

This application uses Linera's multi-chain messaging system with four distinct chain types:

1. **Master Chain**: Administrative operations
    - Mints tokens (`BankrollOperation::MintToken`, `BlackjackOperation::MintToken`)
    - Adds play chains (`BlackjackOperation::AddPlayChain`)
    - Must authorize critical operations via chain ID validation

2. **Public Chains**: Message routing and discovery
    - Players send `FindPlayChain` messages to discover available game chains
    - Routes `AddPlayChain` messages to register new play chains
    - Acts as a directory service

3. **Play Chains**: Game execution environment
    - Hosts active blackjack games with up to 3 players (`MAX_BLACKJACK_PLAYERS`)
    - Manages table seats and game rounds
    - Broadcasts game state via event streams (`BLACKJACK_STREAM_NAME`)

4. **User Chains**: Individual player state
    - Stores user status (`UserStatus` enum in `abi/src/blackjack.rs`)
    - Maintains connection to assigned play chain
    - Handles subscribe/unsubscribe operations

### Contract and Service Pattern

Each application follows Linera's contract-service architecture:

- **Contract** (`contract.rs`): Executes operations that modify blockchain state
    - Processes `Operation` types (e.g., `BlackjackOperation`, `BankrollOperation`)
    - Sends and receives cross-chain messages
    - Emits events for state changes

- **Service** (`service.rs`): Read-only GraphQL query interface
    - Provides queries for frontend applications
    - Does not modify state

### Message Flow Example

Finding and joining a game:

1. User Chain: `FindPlayChain` operation → Public Chain
2. Public Chain: `FindPlayChainResult` message → User Chain (with available chain ID)
3. User Chain: `RequestTableSeat` message → Play Chain
4. Play Chain: `RequestTableSeatResult` message → User Chain
5. User Chain: Subscribe to Play Chain's event stream
6. Play Chain: Broadcasts `GameState` events to all subscribers

### Key State Management

- **Bankroll State** (`bankroll/src/state.rs`):
    - `accounts`: Map of account owners to balances
    - `daily_bonus`: Claimable daily bonus with 24-hour cooldown

- **Blackjack State** (`blackjack/src/state.rs`):
    - `user_status`: Current player status (Idle, FindPlayChain, InGame, etc.)
    - `user_play_chain`: Assigned play chain for the user
    - `channel_game_state`: Current game state from subscribed play chain
    - `profile`: Player betting profile and balance

### GraphQL Integration

Both applications expose GraphQL APIs via the Linera SDK:

- Operations are defined with `#[derive(GraphQLMutationRoot)]`
- Service ABI uses `Request`/`Response` types from async-graphql
- Queries are handled by the service binary, mutations by the contract

## Important Constants and Configuration

- `MAX_BLACKJACK_PLAYERS = 3`: Maximum players per table (defined in `abi/src/blackjack.rs:11`)
- `BLACKJACK_STREAM_NAME = b"blackjack"`: Event stream name for game updates
- `ONE_DAY_CLAIM_DURATION_IN_MICROS = 86_400_000_000`: Daily bonus claim cooldown (in `bankroll/src/lib.rs:75`)
- Rust toolchain: `1.86.0` (see `rust-toolchain.toml`)
- Linera SDK version: `0.15.4` (see `Cargo.toml`)

## WebAssembly Compilation

All contract and service binaries must compile to `wasm32-unknown-unknown`:

- Use `#![cfg_attr(target_arch = "wasm32", no_main)]` attribute in contract/service files
- Custom random number generation required (`getrandom` with custom feature)
- No standard library threading or file I/O available

## Cross-Application Calls

The blackjack application calls into the bankroll application:

- Uses `ApplicationId<BankrollAbi>` in `BlackjackParameters`
- Calls `BankrollOperation::Balance` and `BankrollOperation::UpdateBalance`
- Response types: `BankrollResponse::Balance(Amount)` or `BankrollResponse::Ok`

Example pattern in `blackjack/src/contract.rs`:

```rust
let balance_response = self .runtime
.call_application(/* bankroll operation */)
.await;
```
