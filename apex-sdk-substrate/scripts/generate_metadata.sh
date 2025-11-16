#!/bin/bash
# Script to generate typed metadata for Substrate chains using subxt codegen
#
# This script fetches metadata from a Substrate node and generates Rust types
# that provide compile-time type safety for transactions and storage queries.

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Substrate Metadata Generator${NC}"
echo "=============================="
echo ""

# Check if subxt-cli is installed
if ! command -v subxt &> /dev/null; then
    echo -e "${YELLOW}subxt CLI not found. Installing...${NC}"
    cargo install subxt-cli
fi

# Default chain endpoints
POLKADOT_ENDPOINT="wss://rpc.polkadot.io"
KUSAMA_ENDPOINT="wss://kusama-rpc.polkadot.io"
WESTEND_ENDPOINT="wss://westend-rpc.polkadot.io"

# Parse arguments
CHAIN="${1:-westend}"
OUTPUT_DIR="src/metadata"

case "$CHAIN" in
    polkadot)
        ENDPOINT="$POLKADOT_ENDPOINT"
        OUTPUT_FILE="polkadot.rs"
        ;;
    kusama)
        ENDPOINT="$KUSAMA_ENDPOINT"
        OUTPUT_FILE="kusama.rs"
        ;;
    westend)
        ENDPOINT="$WESTEND_ENDPOINT"
        OUTPUT_FILE="westend.rs"
        ;;
    *)
        # Custom endpoint
        ENDPOINT="$CHAIN"
        OUTPUT_FILE="custom.rs"
        ;;
esac

echo -e "Chain: ${GREEN}$CHAIN${NC}"
echo -e "Endpoint: ${GREEN}$ENDPOINT${NC}"
echo -e "Output: ${GREEN}$OUTPUT_DIR/$OUTPUT_FILE${NC}"
echo ""

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

echo -e "${YELLOW}Fetching metadata and generating Rust types from $ENDPOINT...${NC}"

# Try direct codegen from URL first (more reliable for version compatibility)
subxt codegen \
    --url "$ENDPOINT" \
    > "$OUTPUT_DIR/$OUTPUT_FILE" 2>&1 && {
        echo -e "${GREEN}✓${NC} Metadata fetched and types generated successfully"
    } || {
        # Fallback: try fetching metadata first, then codegen
        echo -e "${YELLOW}Direct codegen failed, trying two-step process...${NC}"
        
        subxt metadata \
            --url "$ENDPOINT" \
            --format bytes \
            > "$OUTPUT_DIR/${CHAIN}_metadata.scale" 2>/dev/null || {
                echo -e "${RED}Failed to fetch metadata from $ENDPOINT${NC}"
                echo -e "${YELLOW}Please check your internet connection and ensure the endpoint is accessible.${NC}"
                exit 1
            }

        echo -e "${GREEN}✓${NC} Metadata fetched successfully"
        echo -e "${YELLOW}Generating Rust types...${NC}"

        subxt codegen \
            --file "$OUTPUT_DIR/${CHAIN}_metadata.scale" \
            > "$OUTPUT_DIR/$OUTPUT_FILE" || {
                echo -e "${RED}Failed to generate Rust types${NC}"
                echo -e "${YELLOW}This might be due to metadata version incompatibility.${NC}"
                echo -e "${YELLOW}Your subxt-cli version: 0.44.0${NC}"
                echo -e "${YELLOW}The chain might be using a newer metadata version.${NC}"
                echo ""
                echo -e "${YELLOW}Possible solutions:${NC}"
                echo "  1. Update subxt-cli: cargo install subxt-cli --force"
                echo "  2. Use a compatible chain endpoint with metadata v14"
                echo "  3. Wait for subxt to support newer metadata versions"
                exit 1
            }

        echo -e "${GREEN}✓${NC} Rust types generated successfully"
    }

echo -e "${GREEN}✓${NC} Rust types generated successfully"
echo ""
echo -e "${GREEN}Done!${NC} Typed metadata saved to: $OUTPUT_DIR/$OUTPUT_FILE"
echo ""
echo "To use the generated types in your code:"
echo "  1. Add to src/metadata/mod.rs:"
echo "     pub mod ${CHAIN};"
echo "  2. Use in your code:"
echo "     use crate::metadata::${CHAIN}::runtime_types;"
echo ""
echo "Example usage:"
echo "  let tx = ${CHAIN}::tx().balances().transfer_keep_alive(dest, amount);"
