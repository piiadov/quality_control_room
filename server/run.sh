#!/bin/bash
# Run Quality Control Room server
# Usage: ./run.sh [debug|release] [config.yaml]

set -e

MODE="${1:-debug}"
CONFIG="${2:-config.yaml}"

cd "$(dirname "$0")"

# Set library path for xgbwrapper
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:+$LD_LIBRARY_PATH:}../lib"

case "$MODE" in
    debug|dev)
        echo "Building debug..."
        cargo build
        echo ""
        echo "Running debug server..."
        exec ./target/debug/server "$CONFIG"
        ;;
    release|prod)
        echo "Building release..."
        cargo build --release
        echo ""
        echo "Running release server..."
        exec ./target/release/server "$CONFIG"
        ;;
    *)
        echo "Usage: $0 [debug|release] [config.yaml]"
        echo ""
        echo "Modes:"
        echo "  debug   - HTTP only (no TLS required)"
        echo "  release - HTTPS required (TLS must be configured)"
        exit 1
        ;;
esac
