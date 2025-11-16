#!/usr/bin/env bash
# shellcheck disable=SC2181
# shellcheck disable=SC2145
# shellcheck disable=SC2034

set -eu

#eval "$(linera net helper)"
#linera_spawn linera net up --with-faucet
#
#export LINERA_FAUCET_URL=http://localhost:8080
#linera wallet init --faucet="$LINERA_FAUCET_URL"
#linera wallet request-chain --faucet="$LINERA_FAUCET_URL"

export PATH="$PWD/target/debug:$PATH"
source /dev/stdin <<<"$(linera net helper 2>/dev/null)"
linera_spawn linera net up --initial-amount 1000000000000 --with-faucet --faucet-port 8080 --faucet-amount 1000000000

# -----------------------------------------------------------------------------------------------------------------
# Build and publish your backend
# -----------------------------------------------------------------------------------------------------------------

FAUCET_URL=http://localhost:8080
GRAPHQL_URL=http://localhost:8081
PUBLIC_CHAIN_AMOUNT=1
PLAY_CHAIN_AMOUNT_FOR_EACH_PUBLIC_CHAIN=1
TOKEN_AMOUNT_TO_MINT=1000000000

# ----------------------------------------------------------
# [FUNCTION] Initiate New Wallet from Faucet
# ----------------------------------------------------------

initiate_new_wallet_from_faucet() {
  linera wallet init --faucet "$FAUCET_URL"
  if [ $? -ne 0 ]; then
      echo "Initiate New Wallet from Faucet failed. Exiting..."
      exit 1
  fi
}

# ----------------------------------------------------------
# [FUNCTION] Open Chain from Faucet
# ----------------------------------------------------------

open_chain_from_faucet() {
  linera wallet request-chain --faucet "$FAUCET_URL"
  if [ $? -ne 0 ]; then
      echo "Open Chain from Faucet failed. Exiting..."
      exit 1
  fi
}

# ----------------------------------------------------------
# Create Initial Default Wallet and User Wallet
# ----------------------------------------------------------
#linera wallet init --faucet "$FAUCET_URL"
#linera sync && linera query-balance

INITIATE_WALLET=$(initiate_new_wallet_from_faucet)

OPEN_NEW_DEFAULT_WALLET=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET"
DEFAULT_CHAIN_ID=${StringArray[0]}

linera sync && linera query-balance

OPEN_NEW_USER_WALLET=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET"
USER_CHAIN_ID=${StringArray[0]}

OPEN_NEW_USER_WALLET_2=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_2"
USER_CHAIN_ID_2=${StringArray[0]}

OPEN_NEW_USER_WALLET_3=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_3"
USER_CHAIN_ID_3=${StringArray[0]}

OPEN_NEW_USER_WALLET_4=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_4"
USER_CHAIN_ID_4=${StringArray[0]}

OPEN_NEW_USER_WALLET_5=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_5"
USER_CHAIN_ID_5=${StringArray[0]}

OPEN_NEW_USER_WALLET_6=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_6"
USER_CHAIN_ID_6=${StringArray[0]}

OPEN_NEW_USER_WALLET_7=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_7"
USER_CHAIN_ID_7=${StringArray[0]}

OPEN_NEW_USER_WALLET_8=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_8"
USER_CHAIN_ID_8=${StringArray[0]}

# ----------------------------------------------------------
# Open New Chain IDs
# ----------------------------------------------------------
PUBLIC_CHAIN_IDS=()
for _ in $(seq 1 $PUBLIC_CHAIN_AMOUNT)
do
  OPEN_NEW_CHAIN=$(open_chain_from_faucet)
  mapfile -t StringArray <<< "$OPEN_NEW_CHAIN"
  NEW_CHAIN_ID=${StringArray[0]}
  PUBLIC_CHAIN_IDS+=("$NEW_CHAIN_ID")
  sleep 1
done

# Convert Chain IDs array to a JSON-formatted list
JSON_PUBLIC_CHAIN_IDS=$(printf '"%s",' "${PUBLIC_CHAIN_IDS[@]}")
JSON_PUBLIC_CHAIN_IDS="[${JSON_PUBLIC_CHAIN_IDS%,}]"

linera sync && linera query-balance

# Echo the values
echo ""
echo "PUBLIC_CHAIN_IDS: ${PUBLIC_CHAIN_IDS[@]}"
echo ""
echo "JSON_PUBLIC_CHAIN_IDS: $JSON_PUBLIC_CHAIN_IDS"
echo ""

# ----------------------------------------------------------
# [FUNCTION] Deploy Bankroll App
# ----------------------------------------------------------
deploy_bankroll_app() {
  linera --wait-for-outgoing-messages project publish-and-create . bankroll \
  --json-parameters "{
  \"master_chain\": \"$DEFAULT_CHAIN_ID\",
  \"bonus\": \"25000\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create Bankroll app failed. Exiting..."
      exit 1
  fi
}

BANKROLL_APP_ID=$(deploy_bankroll_app)
sleep 5

# ----------------------------------------------------------
# [FUNCTION] Deploy BlackJack App
# ----------------------------------------------------------
deploy_black_jack_app() {
  linera --wait-for-outgoing-messages project publish-and-create . blackjack \
  --required-application-ids "$BANKROLL_APP_ID" \
  --json-argument "10000" \
  --json-parameters "{
  \"master_chain\": \"$DEFAULT_CHAIN_ID\",
  \"public_chains\": $JSON_PUBLIC_CHAIN_IDS,
  \"bankroll\": \"$BANKROLL_APP_ID\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create Blackjack app failed. Exiting..."
      exit 1
  fi
}

BLACK_JACK_APP_ID=$(deploy_black_jack_app)

# ----------------------------------------------------------
# Loop through each ChainID to create Play Chains for each Public Chain
# ----------------------------------------------------------

# Associative array to store player arrays by chain_id
declare -A PLAY_CHAIN_ID_COLLECTION

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  # Open New Chain IDs
  # Each Play Chain will have its own designated Public Chain
  PLAY_CHAIN_IDS=()
  for _ in $(seq 1 $PLAY_CHAIN_AMOUNT_FOR_EACH_PUBLIC_CHAIN)
  do
    OPEN_NEW_CHAIN=$(open_chain_from_faucet)
    mapfile -t StringArray <<< "$OPEN_NEW_CHAIN"
    NEW_CHAIN_ID=${StringArray[0]}
    PLAY_CHAIN_IDS+=("$NEW_CHAIN_ID")
    sleep 1
  done

  # Store the array as a space-separated string in the associative array, i.e.
  # Public_Chain_A : [ Play_Chain_A1, Play_Chain_A2, ... ]
  # Public_Chain_B : [ Play_Chain_B1, Play_Chain_B2, ... ]
  # Public_Chain_C : [ Play_Chain_C1, Play_Chain_C2, ... ]
  PLAY_CHAIN_ID_COLLECTION["$PUBLIC_CHAIN_ID"]="${PLAY_CHAIN_IDS[*]}"

  linera sync && linera query-balance
  sleep 1
done

# ----------------------------------------------------------
# Run Node Service in the background
# ----------------------------------------------------------

linera service --port 8081 &
SERVICE_PID=$!

sleep 3
echo "Node service started with PID $SERVICE_PID"
sleep 2

# ----------------------------------------------------------
# Loop through each argument to add Play Chains to each Public Chain
# ----------------------------------------------------------
echo ""
echo "------------------------------------------------"
echo "1 | Add Play Chains to each Public Chain"
echo "------------------------------------------------"
echo ""

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  echo "Processing ChainID: $PUBLIC_CHAIN_ID"
  IFS=' ' read -r -a PLAY_CHAIN_IDS <<< "${PLAY_CHAIN_ID_COLLECTION[$PUBLIC_CHAIN_ID]}"

  for PLAY_CHAIN in "${PLAY_CHAIN_IDS[@]}"; do
    echo "PLAY_CHAIN: $PLAY_CHAIN"

    # Build the GraphQL mutation
    MUTATION="mutation { addPlayChain ( targetPublicChain: \\\"$PUBLIC_CHAIN_ID\\\", playChainId: \\\"$PLAY_CHAIN\\\" ) }"

    # Send request
    curl -s -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID/applications/$BLACK_JACK_APP_ID" \
      -H "Content-Type: application/json" \
      -d "{\"query\":\"$MUTATION\"}" \
      | jq .

    sleep 2
  done

  sleep 2
done

# ----------------------------------------------------------
# Loop through each argument to MintToken to each Public Chain
# ----------------------------------------------------------
echo ""
echo "------------------------------------------------"
echo "2 | MintToken to each Public Chain"
echo "------------------------------------------------"
echo ""

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  echo "MintToken - Processing ChainID: $PUBLIC_CHAIN_ID"

  # Build the GraphQL mutation
  MUTATION="mutation { mintToken ( chainId: \\\"$PUBLIC_CHAIN_ID\\\", amount: \\\"$TOKEN_AMOUNT_TO_MINT\\\" ) }"

  # Send request
  curl -s -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID/applications/$BLACK_JACK_APP_ID" \
    -H "Content-Type: application/json" \
    -d "{\"query\":\"$MUTATION\"}" \
    | jq .

  sleep 2
done

# -----------------------------------------------------------------------------------------------------------------
# Generate config.json for frontend
# -----------------------------------------------------------------------------------------------------------------

echo "Generating config.json for frontend..."

jq -n \
  --arg nodeServiceURL "$GRAPHQL_URL" \
  --arg blackjackAppId "$BLACK_JACK_APP_ID" \
  --arg bankrollAppId "$BANKROLL_APP_ID" \
  --arg conwayDefaultChain "$DEFAULT_CHAIN_ID" \
  --arg conwayUserChain1 "$USER_CHAIN_ID" \
  --arg conwayUserChain2 "$USER_CHAIN_ID_2" \
  --arg conwayUserChain3 "$USER_CHAIN_ID_3" \
  --arg conwayUserChain4 "$USER_CHAIN_ID_4" \
  --arg conwayUserChain5 "$USER_CHAIN_ID_5" \
  --arg conwayUserChain6 "$USER_CHAIN_ID_6" \
  --arg conwayUserChain7 "$USER_CHAIN_ID_7" \
  --arg conwayUserChain8 "$USER_CHAIN_ID_8" \
  '{
    nodeServiceURL: $nodeServiceURL,
    blackjackAppId: $blackjackAppId,
    bankrollAppId: $bankrollAppId,
    conwayDefaultChain: $conwayDefaultChain,
    conwayUserChain1: $conwayUserChain1,
    conwayUserChain2: $conwayUserChain2,
    conwayUserChain3: $conwayUserChain3,
    conwayUserChain4: $conwayUserChain4,
    conwayUserChain5: $conwayUserChain5,
    conwayUserChain6: $conwayUserChain6,
    conwayUserChain7: $conwayUserChain7,
    conwayUserChain8: $conwayUserChain8
  }' > "frontend/web/config.json"

echo "✓ config.json created at frontend/web/config.json"
echo ""

# -----------------------------------------------------------------------------------------------------------------
# Build and run your frontend, if any
# -----------------------------------------------------------------------------------------------------------------

