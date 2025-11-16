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
├── abi/                    # Shared types and game logic library
│   └── src/
│       ├── lib.rs
│       ├── bet_chip_profile.rs
│       ├── blackjack.rs
│       ├── deck.rs
│       ├── player_dealer.rs
│       ├── poker.rs
│       └── random.rs
├── bankroll/               # Token management application
│   └── src/
│       ├── lib.rs
│       ├── contract.rs     # Balance operations, daily bonus
│       ├── service.rs      # GraphQL query interface
│       └── state.rs
├── blackjack/              # Main blackjack game application
│   └── src/
│       ├── lib.rs
│       ├── contract.rs     # Game operations, chain messaging
│       ├── service.rs      # GraphQL query and mutation interface
│       └── state.rs
├── frontend/               # Web frontend (Flutter)
│   └── web/
│       ├── index.html
│       ├── assets/
│       ├── canvaskit/
│       └── ...
├── Dockerfile              # Docker configuration
├── compose.yaml            # Docker Compose configuration
├── run.bash                # Deployment script
├── Makefile                # Build automation
├── Cargo.toml              # Workspace configuration
└── rust-toolchain.toml     # Rust version specification
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

## Running with Docker

The easiest way to get started is using Docker:

1. Clone this repository and navigate to the folder:
   ```bash
   git clone https://github.com/hasToDev/microcard.git
   cd microcard
   ```

2. Start the application with Docker Compose:
   ```bash
   docker compose up -d --build
   ```

3. Monitor the logs to ensure the application is ready:
   ```bash
   docker compose logs -f blackjack
   ```

4. Wait until you see the following message in the logs:
   ```
   Blackjack on Microchains READY!
   ```

5. Open your browser and navigate to [http://localhost:5173](http://localhost:5173) to play the game.

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

## Single Player Game Flow

The single-player game follows this flow:

1. **Start Game**: Player launches the Blackjack application
2. **Balance Check**: The Blackjack app calls the Bankroll application to:
    - Retrieve the latest balance
    - Automatically claim daily rewards if available
3. **Place Bet**: Player places their bet using the available balance
4. **Deal Cards**: When the player deals:
    - The bet amount is deducted from the player's balance
    - Tokens are moved into a blackjack token pool
5. **Game Outcomes**:
    - **Player Loses**: Tokens from the pool are transferred to the Public chain via the Bankroll app
    - **Draw**: Tokens from the pool are returned to the player's balance
    - **Player Wins**:
        - The Bankroll app creates a Debt Record
        - The Debt Record is sent to the Public chain for processing
        - The Public chain sends tokens from its own pool to settle the debt
        - All debt settlement happens automatically in the smart contract
        - Players can continue playing without manual intervention

## Project Status

This project is actively developed and demonstrates Linera's multi-chain capabilities. It includes:

- ✅ Single-player blackjack
- ✅ Token-based betting system
- ✅ Daily bonus rewards
- ✅ Real-time game state updates
- ✅ Cross-application messaging
- ✅ GraphQL API
- 🚧 Multi-player blackjack (under development)
- 🚧 Leaderboard (under development)
- 🚧 Prediction (under development)

## Support

For issues, questions, or contributions, please [open an issue](https://github.com/hasToDev/microcard/issues).

---

Built with ❤️ on [Linera](https://linera.io)
