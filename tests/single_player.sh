#!/bin/bash

# Check if target time is provided
if [ -z "$3" ]; then
    echo "Usage: $0 <LOCAL_NETWORK_URL> <TEST_CHAIN_ID> <BLACK_JACK_APP_ID>"
    exit 1
fi

LOCAL_NETWORK_URL="$1"
TEST_CHAIN_ID="$2"
BLACK_JACK_APP_ID="$3"

# GraphQL endpoint URL
GRAPHQL_ENDPOINT="$LOCAL_NETWORK_URL/chains/$TEST_CHAIN_ID/applications/$BLACK_JACK_APP_ID"

# -------------------------------------------------------------------

echo "CLI | Single Player TEST"
echo "---------------------------"

# -------------------------------------------------------------------

START_GAME_MUTATION="mutation { startSinglePlayerGame }"

echo "Start Game"

curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"$START_GAME_MUTATION\"}" \
  | jq .

sleep 1

# -------------------------------------------------------------------

BET_MUTATION="mutation { bet ( amount: \\\"1000\\\" ) }"

echo "Bet"

# Make HTTP POST request to GraphQL endpoint
# Replace headers as needed for your specific service
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"$BET_MUTATION\"}" \
  | jq .

sleep 1

# -------------------------------------------------------------------

DEAL_MUTATION="mutation { deal }"

echo "Deal"

# Make HTTP POST request to GraphQL endpoint
# Replace headers as needed for your specific service
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"$DEAL_MUTATION\"}" \
  | jq .

sleep 1

# -------------------------------------------------------------------

GET_USER_STATUS_QUERY="query { getUserStatus }"

echo "User Status"

# Make HTTP POST request to GraphQL endpoint
# Replace headers as needed for your specific service
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"$GET_USER_STATUS_QUERY\"}" \
  | jq .

sleep 1

# -------------------------------------------------------------------

GET_PROFILE_QUERY="query { getProfile { seat balance betData { minBet maxBet chipset { amount text enable } } } }"

echo "Profile"

# Make HTTP POST request to GraphQL endpoint
# Replace headers as needed for your specific service
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"$GET_PROFILE_QUERY\"}" \
  | jq .

sleep 1

# -------------------------------------------------------------------

echo "---------------------------"
echo "Test Finished!"
exit 0