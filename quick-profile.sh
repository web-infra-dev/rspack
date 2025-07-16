#!/bin/bash
# cspell:ignore rspack

# Quick profiling for threejs performance regression
set -e

echo "🔍 Quick Rspack Performance Analysis"
echo "===================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this from the rspack root directory"
    exit 1
fi

# Create results directory
mkdir -p profiling-results

# Install flamegraph if not available
if ! command -v flamegraph >/dev/null 2>&1; then
    echo "📦 Installing flamegraph..."
    cargo install flamegraph
fi

# Build with debug symbols for profiling
echo "🔨 Building with debug symbols..."
export CARGO_PROFILE_RELEASE_DEBUG=true
cd crates/node_binding
npm run build:release
cd ../..

# Profile the threejs benchmark specifically
echo "🎯 Profiling threejs development build..."
cd examples/basic

# Clean previous builds
rm -rf dist node_modules/.cache

# Profile with flamegraph
echo "🔥 Generating CPU flamegraph..."
flamegraph --freq 997 --min-width 0.01 -o ../../profiling-results/threejs-cpu.svg -- \
    rspack build --mode development

# Run multiple times to get consistent timing
echo "⏱️  Running timing analysis..."
echo "=== Timing Results ===" > ../../profiling-results/timing-analysis.txt

for i in {1..3}; do
    echo "Run $i:" >> ../../profiling-results/timing-analysis.txt
    rm -rf dist node_modules/.cache
    
    start_time=$(date +%s%N)
    rspack build --mode development --silent
    end_time=$(date +%s%N)
    
    duration=$(( (end_time - start_time) / 1000000 ))
    echo "  Build time: ${duration}ms" >> ../../profiling-results/timing-analysis.txt
done

# Get bundle stats
echo "📊 Analyzing bundle composition..."
rspack build --mode development --json > ../../profiling-results/webpack-stats.json

# Basic analysis if jq is available
if command -v jq >/dev/null 2>&1; then
    echo "=== Bundle Analysis ===" >> ../../profiling-results/bundle-summary.txt
    echo "Build time: $(jq -r '.time' ../../profiling-results/webpack-stats.json)ms" >> ../../profiling-results/bundle-summary.txt
    echo "Total modules: $(jq -r '.modules | length' ../../profiling-results/webpack-stats.json)" >> ../../profiling-results/bundle-summary.txt
    echo "Total assets: $(jq -r '.assets | length' ../../profiling-results/webpack-stats.json)" >> ../../profiling-results/bundle-summary.txt
    
    # Find largest modules
    echo "=== Largest Modules ===" >> ../../profiling-results/bundle-summary.txt
    jq -r '.modules | sort_by(.size) | reverse | .[0:5] | .[] | "\(.size) bytes - \(.name)"' \
        ../../profiling-results/webpack-stats.json >> ../../profiling-results/bundle-summary.txt
fi

cd ../..

echo ""
echo "✅ Quick profiling complete!"
echo ""
echo "📁 Results saved to profiling-results/:"
echo "  🔥 threejs-cpu.svg - CPU flamegraph (open in browser)"
echo "  ⏱️  timing-analysis.txt - Build timing results"
echo "  📊 webpack-stats.json - Detailed webpack stats"
echo "  📋 bundle-summary.txt - Bundle analysis summary"
echo ""
echo "🔍 Next steps:"
echo "  1. Open threejs-cpu.svg in browser to identify hot functions"
echo "  2. Compare timing with baseline builds"
echo "  3. Look for wide bars in flamegraph indicating bottlenecks"