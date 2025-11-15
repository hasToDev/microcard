#!/bin/bash
# shellcheck disable=SC2181
# shellcheck disable=SC2145

# Check if three values are provided
if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <FAUCET_URL> <GRAPHQL_URL> <LOCAL_NETWORK_URL>"
  exit 1
fi

start=$(date +%s%3N)

FAUCET_URL=$1
GRAPHQL_URL=$2
LOCAL_NETWORK_URL=$3
PUBLIC_CHAIN_AMOUNT=1
PLAY_CHAIN_AMOUNT_FOR_EACH_PUBLIC_CHAIN=1

LINERA_TMP_DIR=/home/hasto/.config/linera
TOKEN_AMOUNT_TO_MINT=1000000000

# ----------------------------------------------------------
# Clear current wallet
# ----------------------------------------------------------
rm -rf cd $LINERA_TMP_DIR
mkdir $LINERA_TMP_DIR

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

# shellcheck disable=SC2034
INITIATE_WALLET=$(initiate_new_wallet_from_faucet)

OPEN_NEW_DEFAULT_WALLET=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET"
DEFAULT_CHAIN_ID=${StringArray[0]}

linera sync && linera query-balance

OPEN_NEW_USER_WALLET=$(open_chain_from_faucet)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET"
USER_CHAIN_ID=${StringArray[0]}

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

# ----------------------------------------------------------
# Stop Node Service
# ----------------------------------------------------------
echo "Stopping service..."
kill $SERVICE_PID

# ------------------------------------------------------------
# Show Bankroll App ID, BlackJack App ID, Default Chain ID, User Chain ID
# ------------------------------------------------------------
echo "BLACKJACK APP ID:"
echo "$BLACK_JACK_APP_ID"
echo ""
echo "BANKROLL APP ID:"
echo "$BANKROLL_APP_ID"
echo ""
echo "DEFAULT CHAIN ID:"
echo "$DEFAULT_CHAIN_ID"
echo ""
echo "USER CHAIN ID:"
echo "$USER_CHAIN_ID"
echo ""
echo "BLACKJACK:"
echo "$LOCAL_NETWORK_URL/chains/$LOCAL_NETWORK_URL/applications/$BLACK_JACK_APP_ID"
echo ""
echo "$LOCAL_NETWORK_URL/chains/$USER_CHAIN_ID/applications/$BLACK_JACK_APP_ID"
echo ""
echo "BANKROLL:"
echo "$LOCAL_NETWORK_URL/chains/$LOCAL_NETWORK_URL/applications/$BANKROLL_APP_ID"
echo ""
echo "$LOCAL_NETWORK_URL/chains/$USER_CHAIN_ID/applications/$BANKROLL_APP_ID"
echo ""
echo "./tests/single_player.sh $LOCAL_NETWORK_URL $LOCAL_NETWORK_URL $BLACK_JACK_APP_ID"
echo ""

end=$(date +%s%3N)
total_ms=$(( end - start ))
ms=$(( total_ms % 1000 ))
seconds=$(( total_ms / 1000 ))
printf "Total Runtime: %d seconds and %d ms\n" $seconds $ms
echo ""