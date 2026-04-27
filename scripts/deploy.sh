#!/bin/bash

# Exit on error
set -e

# Default values
NETWORK="testnet"
RPC_URL=""
NETWORK_PASSPHRASE=""
ADMIN_SECRET=""
ADMIN_ADDRESS=""

# Load existing .env if present
if [ -f .env ]; then
    source .env
fi

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --network) NETWORK="$2"; shift ;;
        --admin) ADMIN_IDENTITY="$2"; shift ;;
        --help)
            echo "Usage: ./scripts/deploy.sh [OPTIONS]"
            echo "Options:"
            echo "  --network <network>   Network to deploy to (testnet/mainnet). Default: testnet"
            echo "  --admin <identity>    Name of the stellar-cli identity used to deploy and initialize the contracts."
            echo "                        (MANDATORY) Create it first using: stellar keys add <identity>"
            exit 0
            ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

# Set network variables
if [ "$NETWORK" == "testnet" ]; then
    RPC_URL="https://soroban-testnet.stellar.org:443"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
elif [ "$NETWORK" == "mainnet" ]; then
    RPC_URL="https://soroban-rpc.mainnet.stellar.org:443"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
else
    echo "Error: Unknown network '$NETWORK'. Use 'testnet' or 'mainnet'."
    exit 1
fi

if [ -z "$ADMIN_IDENTITY" ]; then
    echo "Error: --admin <identity> is required to deploy and initialize."
    echo "Create it first using: stellar keys add <identity>"
    exit 1
fi

echo "==========================================="
echo "Building and Deploying InheritX Contracts"
echo "Network: $NETWORK"
echo "RPC URL: $RPC_URL"
echo "Admin Identity: $ADMIN_IDENTITY"
echo "==========================================="

# Setup identity for deployment
# We DO need the public address for initialization
ADMIN_ADDRESS=$(stellar keys address "$ADMIN_IDENTITY" 2>/dev/null)

if [ -z "$ADMIN_ADDRESS" ]; then
    echo "Error: Could not derive public address for identity '$ADMIN_IDENTITY'."
    echo "Make sure the identity exists: stellar keys ls"
    exit 1
fi

echo "Admin Address: $ADMIN_ADDRESS"

# Try to fund the account if it's on a test network
if [ "$NETWORK" == "testnet" ] || [ "$NETWORK" == "futurenet" ]; then
    echo "Ensuring the account is funded on $NETWORK..."
    stellar keys fund "$ADMIN_IDENTITY" --network "$NETWORK" > /dev/null 2>&1 || true
fi

# 1. Build Contracts
echo "[1/4] Building contracts..."

# Navigate to contracts dir if running from root
if [ -d "contracts" ]; then
    pushd contracts > /dev/null
elif [ -d "../contracts" ]; then
    pushd ../contracts > /dev/null
else
    echo "Error: Could not find contracts directory"
    exit 1
fi

stellar contract build

# Optimize builds using stellar-cli
echo "[2/4] Optimizing contracts..."
stellar contract build --package example-contract --optimize
stellar contract build --package inheritance-contract --optimize

# 2. Deploy Contracts
echo "[3/4] Deploying contracts..."

echo "- Deploying example-contract..."
EXAMPLE_CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32v1-none/release/example_contract.wasm \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --source-account "$ADMIN_IDENTITY")
echo "  ↳ ID: $EXAMPLE_CONTRACT_ID"

echo "- Deploying inheritance-contract..."
INHERITANCE_CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32v1-none/release/inheritance_contract.wasm \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --source-account "$ADMIN_IDENTITY")
echo "  ↳ ID: $INHERITANCE_CONTRACT_ID"

# Return to original directory
popd > /dev/null

# 3. Initialize inheritance-contract
echo "[4/4] Initializing inheritance-contract..."
stellar contract invoke \
    --id "$INHERITANCE_CONTRACT_ID" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --source-account "$ADMIN_IDENTITY" \
    -- \
    initialize_admin \
    --admin "$ADMIN_ADDRESS"

echo "  ↳ Initialized successfully."

# 4. Save to .env
echo "Saving addresses to .env..."

# Ensure file exists
touch .env

# Remove old addresses if they exist
grep -v "^EXAMPLE_CONTRACT_ID=" .env > .env.tmp && mv .env.tmp .env
grep -v "^INHERITANCE_CONTRACT_ID=" .env > .env.tmp && mv .env.tmp .env

# Add new addresses
echo "EXAMPLE_CONTRACT_ID=$EXAMPLE_CONTRACT_ID" >> .env
echo "INHERITANCE_CONTRACT_ID=$INHERITANCE_CONTRACT_ID" >> .env

echo "==========================================="
echo "Deployment Complete!"
echo "Check .env file for the new contract IDs."
echo "==========================================="
