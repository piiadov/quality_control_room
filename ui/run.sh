#!/bin/bash
# Run Quality Control Room UI
# Usage: ./run.sh [dev|build|preview]

set -e

MODE="${1:-dev}"

cd "$(dirname "$0")"

case "$MODE" in
    dev|debug)
        echo "Starting development server..."
        npm run dev
        ;;
    build)
        echo "Building for production..."
        npm run build
        ;;
    preview|release)
        echo "Building and previewing production build..."
        npm run build
        echo ""
        echo "Starting preview server..."
        npm run preview
        ;;
    *)
        echo "Usage: $0 [dev|build|preview]"
        echo ""
        echo "Modes:"
        echo "  dev      - Development server with hot reload"
        echo "  build    - Build for production"
        echo "  preview  - Build and serve production build"
        exit 1
        ;;
esac
