#!/bin/bash

# Script to build rspack CLI and run build
# Usage: ./run-build.sh [build|dev]

set -e

COMMAND=${1:-build}

echo "🔍 Building rspack CLI in development mode..."
echo ""

# Build rspack CLI in development mode first (from parent directory)
echo "🔨 Building rspack CLI using workspace commands..."
cd /Users/bytedance/RustroverProjects/rspack
pnpm build:binding:dev
pnpm build:js
cd examples/basic
echo ""
echo "🔍 Running rspack $COMMAND..."
echo ""

# Run rspack build
echo "📁 Current working directory: $(pwd)"
npx rspack $COMMAND

echo ""
echo "✅ Build completed successfully!"