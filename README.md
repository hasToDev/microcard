# Blackjack on Microchains

<img src="./Blackjack%20on%20Microchains.jpg" width="700">

A decentralized blackjack gaming application built on the Linera blockchain platform. Play blackjack across multiple
microchains with real-time updates, token-based betting, and daily bonuses.

## Overview

This project implements a multi-chain blackjack game where game logic, token management, and player state are
distributed across different chain types. Built with Rust and compiled to WebAssembly, it showcases Linera's unique
multi-chain architecture and cross-application messaging capabilities.

### Key Features

- **Multi-player games**: Up to 3 players per table
- **Single-player mode**: Play against the dealer
- **Token-based economy**: Integrated bankroll system with daily bonuses
- **Real-time updates**: Event streaming for live game state
- **Decentralized**: Game logic runs across multiple microchains
- **GraphQL API**: Full query and mutation support

## Architecture

### Multi-Chain Design

The application uses four distinct chain types:

1. **Master Chain**: Administrative operations
    - Mints tokens for both applications
    - Adds play chains to public chains
    - Requires authorization via chain ID validation

2. **Public Chains**: Message routing and discovery
    - Directory service for play chain discovery
    - Routes `FindPlayChain` requests
    - Multiple public chains supported

3. **Play Chains**: Game execution environment
    - Hosts active blackjack games
    - Supports up to 3 players per table
    - Broadcasts game state via event streams

4. **User Chains**: Individual player state
    - Stores user status (Idle, FindPlayChain, InGame, etc.)
    - Maintains connection to assigned play chain
    - Handles subscribe/unsubscribe operations

### Workspace Structure

The project is organized as a Cargo workspace with three main crates:

```
microcard/
├── abi/              # Shared types and game logic library
│   ├── bet_chip_profile.rs
│   ├── blackjack.rs
│   ├── deck.rs
│   ├── player_dealer.rs
│   ├── poker.rs
│   └── random.rs
├── bankroll/         # Token management application
│   ├── contract.rs   # Balance operations, daily bonus
│   ├── service.rs    # GraphQL query interface
│   └── state.rs
├── blackjack/        # Main blackjack game application
│   ├── contract.rs   # Game operations, chain messaging
│   ├── service.rs    # GraphQL query and mutation interface
│   └── state.rs
├── tests/            # Deployment and test scripts
│   ├── test_run_single_node.sh
│   └── test_run_multi_node.sh
├── Cargo.toml
└── rust-toolchain.toml
```

### Application Dependencies

```
┌─────────────────────────────────────────────┐
│      Blackjack Application                  │
│  (Depends on Bankroll via cross-app calls)  │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│      Bankroll Application                   │
│  (Manages tokens and daily bonuses)         │
└─────────────────────────────────────────────┘

Both depend on:
┌─────────────────────────────────────────────┐
│      ABI Library                            │
│  (Shared game logic and data structures)    │
└─────────────────────────────────────────────┘
```

## Prerequisites

- **Rust**: Version 1.86.0 (specified in `rust-toolchain.toml`)
- **Linera CLI**: Linera SDK 0.15.4
- **wasm32-unknown-unknown** target installed
- **jq**: JSON processor for deployment scripts
- **Linera local network** running with faucet service

### Install Rust and Components

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install components (done automatically via rust-toolchain.toml)
rustup component add clippy rustfmt rust-src
```

### Install Linera CLI

Follow the [Linera installation guide](https://linera.io) to install the Linera CLI tools.

## Building the Project

### Build All Packages

```bash
# Build all workspace members for WebAssembly target
cargo build --release --target wasm32-unknown-unknown
```

### Build Specific Packages

```bash
# Build individual packages
cargo build -p blackjack --release --target wasm32-unknown-unknown
cargo build -p bankroll --release --target wasm32-unknown-unknown
cargo build -p abi --release --target wasm32-unknown-unknown
```

### Development Commands

```bash
# Check code without building
cargo check

# Run clippy for linting
cargo clippy --target wasm32-unknown-unknown

# Format code
cargo fmt
```

## Deployment

> **⚠️ IMPORTANT**: You must deploy applications in the correct order due to dependencies.

### Deployment Order

1. **Deploy Bankroll Application First** (requires Default Chain ID)
2. **Deploy Blackjack Application Second** (requires Bankroll App ID)
3. **Configure Play Chains**
4. **Mint Tokens**

The complete deployment process is automated in `test/test_run_single_node.sh`. Below is a step-by-step guide.

### Step 1: Set Up Linera Local Network

```bash
# Set up local network with faucet
export PATH="$PWD/target/debug:$PATH"
source /dev/stdin <<<"$(linera net helper 2>/dev/null)"
linera_spawn linera net up \
  --initial-amount 1000000000000 \
  --with-faucet \
  --faucet-port 3131 \
  --faucet-amount 1000000000
```

### Step 2: Initialize Wallet

```bash
# Initialize wallet from faucet
linera wallet init --faucet http://localhost:3131

# Request default chain
linera wallet request-chain --faucet http://localhost:3131

# Save default chain ID
DEFAULT_CHAIN_ID=$(linera wallet show | grep "Public Key" -A 1 | tail -n 1 | awk '{print $2}')
```

### Step 3: Create Public and Play Chains

```bash
# Create public chains (for game discovery)
PUBLIC_CHAIN_1=$(linera wallet request-chain --faucet http://localhost:3131)
PUBLIC_CHAIN_2=$(linera wallet request-chain --faucet http://localhost:3131)

# Create play chains (for hosting games)
PLAY_CHAIN_1=$(linera wallet request-chain --faucet http://localhost:3131)
PLAY_CHAIN_2=$(linera wallet request-chain --faucet http://localhost:3131)
# ... create more as needed
```

### Step 4: Deploy Bankroll Application

```bash
# Deploy bankroll (requires DEFAULT_CHAIN_ID)
BANKROLL_APP_ID=$(linera --wait-for-outgoing-messages project publish-and-create . bankroll \
  --json-parameters "{
    \"master_chain\": \"$DEFAULT_CHAIN_ID\",
    \"bonus\": \"25000\"
  }" | grep "Application ID:" | awk '{print $3}')

echo "Bankroll App ID: $BANKROLL_APP_ID"
```

### Step 5: Deploy Blackjack Application

```bash
# Deploy blackjack (requires BANKROLL_APP_ID)
BLACKJACK_APP_ID=$(linera --wait-for-outgoing-messages project publish-and-create . blackjack \
  --required-application-ids "$BANKROLL_APP_ID" \
  --json-argument "10000" \
  --json-parameters "{
    \"master_chain\": \"$DEFAULT_CHAIN_ID\",
    \"public_chains\": [\"$PUBLIC_CHAIN_1\", \"$PUBLIC_CHAIN_2\"],
    \"bankroll\": \"$BANKROLL_APP_ID\"
  }" | grep "Application ID:" | awk '{print $3}')

echo "Blackjack App ID: $BLACKJACK_APP_ID"
```

### Step 6: Start Linera Service

```bash
# Start node service in background
linera service --port 8081 &
```

### Step 7: Configure Play Chains

```bash
# Add play chains to public chains via GraphQL
GRAPHQL_URL="http://localhost:8081"

# Add each play chain to each public chain
curl -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID/applications/$BLACKJACK_APP_ID" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"mutation { addPlayChain(targetPublicChain: \\\"$PUBLIC_CHAIN_1\\\", playChainId: \\\"$PLAY_CHAIN_1\\\") }\"}"
```

### Step 8: Mint Tokens

```bash
# Mint tokens to public chains (1 billion tokens each)
curl -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID/applications/$BLACKJACK_APP_ID" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"mutation { mintToken(chainId: \\\"$PUBLIC_CHAIN_1\\\", amount: \\\"1000000000\\\") }\"}"
```

### Automated Deployment

For a complete automated deployment, use the provided script:

```bash
./test/test_run_single_node.sh \
  http://localhost:3131 \     # FAUCET_URL
  http://localhost:8081 \     # GRAPHQL_URL
  http://localhost:8081       # LOCAL_NETWORK_URL
```

This script will:

- Initialize a wallet
- Create default, user, public, and play chains
- Deploy both Bankroll and Blackjack applications in the correct order
- Configure play chains
- Mint tokens
- Output all important IDs for testing

## Testing

### Single-Node Test

Test the complete deployment on a single node:

```bash
# Set up local network
source /dev/stdin <<<"$(linera net helper 2>/dev/null)"
linera_spawn linera net up --initial-amount 1000000000000 \
  --with-faucet --faucet-port 3131 --faucet-amount 1000000000

# Run deployment test
./test/test_run_single_node.sh \
  http://localhost:3131 \
  http://localhost:8081 \
  http://localhost:8081
```

### Multi-Node Test

Test with multiple players across different nodes:

```bash
./test/test_run_multi_node.sh \
  http://localhost:3131 \
  http://localhost:8081 \
  http://localhost:8081
```

This creates three wallet services:

- Default wallet (port 8081)
- Player A wallet (port 8082)
- Player B wallet (port 8083)

## Game Mechanics

### Daily Bonus System

- Players can claim a daily bonus of **25,000 tokens**
- Bonus has a **24-hour cooldown** (86,400,000,000 microseconds)
- Automatically checked when querying balance

### Betting System

- Configurable min/max bets per table
- 5-chip chipset for betting
- Balance validation before placing bets
- Cross-application balance management with Bankroll

### Multi-Player Games

- Up to **3 players** per table (`MAX_BLACKJACK_PLAYERS`)
- Players request specific seat IDs (0, 1, or 2)
- Real-time game state via event streaming
- Players subscribe to play chain for updates

### Single-Player Games

- Play against the dealer on your user chain
- No need to find or join a play chain
- Instant game start and betting

## Project Status

This project is actively developed and demonstrates Linera's multi-chain capabilities. It includes:

- ✅ Single-player blackjack
- ✅ Multi-player blackjack (up to 3 players)
- ✅ Token-based betting system
- ✅ Daily bonus rewards
- ✅ Real-time game state updates
- ✅ Cross-application messaging
- ✅ GraphQL API
- ✅ Automated deployment scripts
- ✅ Multi-node testing

## License

-

## Contributing

-

## Support

For issues, questions, or contributions, please [open an issue](https://github.com/hasToDev/microcard/issues).

---

Built with ❤️ on [Linera](https://linera.io)
