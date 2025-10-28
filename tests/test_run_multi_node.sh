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
PUBLIC_CHAIN_AMOUNT=2
PLAY_CHAIN_AMOUNT_FOR_EACH_PUBLIC_CHAIN=2

LINERA_TMP_DIR=/home/hasto/.config/linera
DELAY_IN_SECONDS=5
TOKEN_AMOUNT_TO_MINT=1000000000

# ----------------------------------------------------------
# Clear current wallet
# ----------------------------------------------------------
rm -rf cd $LINERA_TMP_DIR
mkdir $LINERA_TMP_DIR

export LINERA_WALLET_1="$LINERA_TMP_DIR/wallet_1.json"
export LINERA_KEYSTORE_1="$LINERA_TMP_DIR/keystore_1.json"
export LINERA_STORAGE_1="rocksdb:$LINERA_TMP_DIR/client_1.db"

export LINERA_WALLET_2="$LINERA_TMP_DIR/wallet_2.json"
export LINERA_KEYSTORE_2="$LINERA_TMP_DIR/keystore_2.json"
export LINERA_STORAGE_2="rocksdb:$LINERA_TMP_DIR/client_2.db"

export LINERA_WALLET_3="$LINERA_TMP_DIR/wallet_3.json"
export LINERA_KEYSTORE_3="$LINERA_TMP_DIR/keystore_3.json"
export LINERA_STORAGE_3="rocksdb:$LINERA_TMP_DIR/client_3.db"

# ----------------------------------------------------------
# [FUNCTION] Initiate New Wallet from Faucet
# ----------------------------------------------------------

initiate_new_wallet_from_faucet() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: initiate_new_wallet_from_faucet <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" wallet init --faucet "$FAUCET_URL"
  if [ $? -ne 0 ]; then
      echo "Initiate New Wallet from Faucet failed. Exiting..."
      exit 1
  fi
}

# ----------------------------------------------------------
# [FUNCTION] Open Chain from Faucet
# ----------------------------------------------------------

open_chain_from_faucet() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: open_chain_from_faucet <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" wallet request-chain --faucet "$FAUCET_URL"
  if [ $? -ne 0 ]; then
      echo "Open Chain from Faucet failed. Exiting..."
      exit 1
  fi
}

# ----------------------------------------------------------
# Create Initial Default Wallet and User Wallet
# ----------------------------------------------------------

# shellcheck disable=SC2034
INITIATE_WALLET_1=$(initiate_new_wallet_from_faucet 1)

OPEN_NEW_DEFAULT_WALLET_1=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET_1"
DEFAULT_CHAIN_ID=${StringArray[0]}

linera --with-wallet 1 sync && linera --with-wallet 1 query-balance

# ----------------------------------------------------------
# [PLAYER A] Create Initial Default Wallet and User Wallet
# ----------------------------------------------------------

# shellcheck disable=SC2034
INITIATE_WALLET_2=$(initiate_new_wallet_from_faucet 2)

OPEN_NEW_DEFAULT_WALLET_2=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET_2"
PLAYER_A_CHAIN_ID=${StringArray[0]}

linera --with-wallet 2 sync && linera --with-wallet 2 query-balance

# ----------------------------------------------------------
# [PLAYER B] Create Initial Default Wallet and User Wallet
# ----------------------------------------------------------

# shellcheck disable=SC2034
INITIATE_WALLET_3=$(initiate_new_wallet_from_faucet 3)

OPEN_NEW_DEFAULT_WALLET_3=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET_3"
PLAYER_B_CHAIN_ID=${StringArray[0]}

linera --with-wallet 3 sync && linera --with-wallet 3 query-balance

# ----------------------------------------------------------
# Open New Chain IDs
# ----------------------------------------------------------
PUBLIC_CHAIN_IDS=()
for _ in $(seq 1 $PUBLIC_CHAIN_AMOUNT)
do
  OPEN_NEW_CHAIN=$(open_chain_from_faucet 1)
  mapfile -t StringArray <<< "$OPEN_NEW_CHAIN"
  NEW_CHAIN_ID=${StringArray[0]}
  PUBLIC_CHAIN_IDS+=("$NEW_CHAIN_ID")
  sleep 1
done

# Convert Chain IDs array to a JSON-formatted list
JSON_PUBLIC_CHAIN_IDS=$(printf '"%s",' "${PUBLIC_CHAIN_IDS[@]}")
JSON_PUBLIC_CHAIN_IDS="[${JSON_PUBLIC_CHAIN_IDS%,}]"

linera --with-wallet 1 sync && linera --with-wallet 1 query-balance

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
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: deploy_bankroll_app <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" --wait-for-outgoing-messages project publish-and-create . bankroll \
  --json-parameters "{
  \"master_chain\": \"$DEFAULT_CHAIN_ID\",
  \"bonus\": \"25000\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create Bankroll app failed. Exiting..."
      exit 1
  fi
}

BANKROLL_APP_ID=$(deploy_bankroll_app 1)
sleep 5

# ----------------------------------------------------------
# [FUNCTION] Deploy BlackJack App
# ----------------------------------------------------------
deploy_black_jack_app() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: deploy_black_jack_app <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" --wait-for-outgoing-messages project publish-and-create . blackjack \
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

BLACK_JACK_APP_ID=$(deploy_black_jack_app 1)

# ----------------------------------------------------------
# Loop through each ChainID to create Play Chains for each Public Chain
# ----------------------------------------------------------

echo ""
echo "------------------------------------------------"
echo "1 | Create Play Chains for each Public Chain"
echo "------------------------------------------------"
echo ""

# Associative array to store player arrays by chain_id
declare -A PLAY_CHAIN_ID_COLLECTION

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  # Open New Chain IDs
  # Each Play Chain will have its own designated Public Chain
  PLAY_CHAIN_IDS=()
  for _ in $(seq 1 $PLAY_CHAIN_AMOUNT_FOR_EACH_PUBLIC_CHAIN)
  do
    OPEN_NEW_CHAIN=$(open_chain_from_faucet 1)
    mapfile -t StringArray <<< "$OPEN_NEW_CHAIN"
    NEW_CHAIN_ID=${StringArray[0]}
    PLAY_CHAIN_IDS+=("$NEW_CHAIN_ID")
    sleep 2
  done

  # Store the array as a space-separated string in the associative array, i.e.
  # Public_Chain_A : [ Play_Chain_A1, Play_Chain_A2, ... ]
  # Public_Chain_B : [ Play_Chain_B1, Play_Chain_B2, ... ]
  # Public_Chain_C : [ Play_Chain_C1, Play_Chain_C2, ... ]
  PLAY_CHAIN_ID_COLLECTION["$PUBLIC_CHAIN_ID"]="${PLAY_CHAIN_IDS[*]}"

  linera --with-wallet 1 sync && linera --with-wallet 1 query-balance
  sleep 2
done

# ----------------------------------------------------------
# Run Node Service in the background
# ----------------------------------------------------------

linera --with-wallet 1 service --port 8081 &
SERVICE_PID=$!

sleep 3
echo "Node service started with PID $SERVICE_PID"
sleep 2

# ----------------------------------------------------------
# Loop through each argument to AddPlayChain to each Public Chain
# ----------------------------------------------------------
echo ""
echo "------------------------------------------------"
echo "2 | Add Play Chains to each Public Chain"
echo "------------------------------------------------"
echo ""

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  echo "AddPlayChain - Processing ChainID: $PUBLIC_CHAIN_ID"
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
echo "3 | MintToken to each Public Chain"
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
sleep $DELAY_IN_SECONDS

# ----------------------------------------------------------
# Prepare for Node Service - CURRENTLY DISABLED
# ----------------------------------------------------------
#echo "Prepare for Node Service..."
#sudo sysctl -w net.ipv4.conf.all.route_localnet=1
#sudo iptables -t nat -A PREROUTING -s 192.168.1.0/24 -p tcp --dport 8081 -j DNAT --to-destination 127.0.0.1:8081
#sudo iptables -t nat -A PREROUTING -s 192.168.1.0/24 -p tcp --dport 8082 -j DNAT --to-destination 127.0.0.1:8082
#sudo iptables -t nat -A PREROUTING -s 192.168.1.0/24 -p tcp --dport 8083 -j DNAT --to-destination 127.0.0.1:8083

# ----------------------------------------------------------
# Running Node Service in Background
# ----------------------------------------------------------
echo "Running Node Service in Background..."
sleep $DELAY_IN_SECONDS
linera --with-wallet 1 service --port 8081 &
sleep $DELAY_IN_SECONDS
linera --with-wallet 2 service --port 8082 &
sleep $DELAY_IN_SECONDS
linera --with-wallet 3 service --port 8083 &
sleep $DELAY_IN_SECONDS

# ------------------------------------------------------------
# Show Bankroll App ID, BlackJack App ID, Default Chain ID, User Chain ID
# ------------------------------------------------------------
DEFAULT_LOCAL_NETWORK_URL="${LOCAL_NETWORK_URL%:*}:8081"

PLAYER_A_GRAPHQL_URL="${GRAPHQL_URL%:*}:8082"
PLAYER_A_LOCAL_NETWORK_URL="${LOCAL_NETWORK_URL%:*}:8082"

PLAYER_B_GRAPHQL_URL="${GRAPHQL_URL%:*}:8083"
PLAYER_B_LOCAL_NETWORK_URL="${LOCAL_NETWORK_URL%:*}:8083"

echo ""
echo "BANKROLL APP ID:"
echo "$BANKROLL_APP_ID"
echo ""
echo "BLACKJACK APP ID:"
echo "$BLACK_JACK_APP_ID"
echo ""
echo "DEFAULT CHAIN ID:"
echo "$DEFAULT_CHAIN_ID"
echo "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID/applications/$BLACK_JACK_APP_ID"
echo "$LOCAL_NETWORK_URL/chains/$DEFAULT_CHAIN_ID/applications/$BLACK_JACK_APP_ID"
echo "./temp/BABBAGE-LOCAL/branch-cross-app/test/single_player.sh $DEFAULT_LOCAL_NETWORK_URL $DEFAULT_CHAIN_ID $BLACK_JACK_APP_ID"
echo ""
echo "PLAYER A CHAIN ID:"
echo "$PLAYER_A_CHAIN_ID"
echo "$PLAYER_A_GRAPHQL_URL/chains/$PLAYER_A_CHAIN_ID/applications/$BLACK_JACK_APP_ID"
echo "$PLAYER_A_LOCAL_NETWORK_URL/chains/$PLAYER_A_CHAIN_ID/applications/$BLACK_JACK_APP_ID"
echo "./temp/BABBAGE-LOCAL/branch-cross-app/test/single_player.sh $PLAYER_A_LOCAL_NETWORK_URL $PLAYER_A_CHAIN_ID $BLACK_JACK_APP_ID"
echo ""
echo "PLAYER B CHAIN ID:"
echo "$PLAYER_B_CHAIN_ID"
echo "$PLAYER_B_GRAPHQL_URL/chains/$PLAYER_B_CHAIN_ID/applications/$BLACK_JACK_APP_ID"
echo "$PLAYER_B_LOCAL_NETWORK_URL/chains/$PLAYER_B_CHAIN_ID/applications/$BLACK_JACK_APP_ID"
echo "./temp/BABBAGE-LOCAL/branch-cross-app/test/single_player.sh $PLAYER_B_LOCAL_NETWORK_URL $PLAYER_B_CHAIN_ID $BLACK_JACK_APP_ID"
echo ""

end=$(date +%s%3N)
total_ms=$(( end - start ))
ms=$(( total_ms % 1000 ))
seconds=$(( total_ms / 1000 ))
printf "Total Runtime: %d seconds and %d ms\n" $seconds $ms
echo ""