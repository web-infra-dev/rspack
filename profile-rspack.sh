#!/bin/bash

# Rspack Performance Profiling Script
set -e

PROFILE_DIR="./profiling-results"
EXAMPLES_DIR="./examples/basic"
BENCHMARK_RUNS=3

echo "🔍 Rspack Performance Profiling Setup"
echo "======================================"

# Create profiling directory
mkdir -p "$PROFILE_DIR"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install profiling tools if needed
setup_profiling_tools() {
    echo "📦 Setting up profiling tools..."
    
    # Install cargo-flamegraph for CPU profiling
    if ! command_exists flamegraph; then
        echo "Installing flamegraph..."
        cargo install flamegraph
    fi
    
    # Install hyperfine for benchmarking
    if ! command_exists hyperfine; then
        echo "Installing hyperfine..."
        if [[ "$OSTYPE" == "darwin"* ]]; then
            brew install hyperfine
        else
            echo "Please install hyperfine manually: https://github.com/sharkdp/hyperfine"
        fi
    fi
    
    # Install perf (Linux only)
    if [[ "$OSTYPE" == "linux-gnu"* ]] && ! command_exists perf; then
        echo "Installing perf..."
        sudo apt-get update && sudo apt-get install -y linux-tools-common linux-tools-generic
    fi
    
    # Install Instruments (macOS only)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "✅ Instruments available via Xcode"
    fi
}

# Build Rspack with profiling symbols
build_with_profiling() {
    echo "🔨 Building Rspack with profiling symbols..."
    
    # Set profiling environment variables
    export CARGO_PROFILE_RELEASE_DEBUG=true
    export RUSTFLAGS="-C target-cpu=native -C link-arg=-Wl,--compress-debug-sections=zlib"
    
    # Build the binding with profiling info
    cd crates/node_binding
    echo "Building node binding with profiling..."
    npm run build:release
    cd ../..
    
    # Build CLI
    echo "Building CLI..."
    npm run build:cli
    
    echo "✅ Build complete with profiling symbols"
}

# Profile threejs benchmark specifically
profile_threejs_benchmark() {
    echo "🎯 Profiling threejs benchmark..."
    
    cd "$EXAMPLES_DIR"
    
    # Clean previous builds
    rm -rf dist node_modules/.cache
    
    echo "Running threejs benchmark with flamegraph..."
    
    # Profile with flamegraph (CPU profiling)
    if command_exists flamegraph; then
        echo "🔥 Generating flamegraph..."
        flamegraph --freq 997 --min-width 0.01 -o "$PROFILE_DIR/threejs-flamegraph.svg" -- \
            rspack build --mode development --config rspack.config.js
    fi
    
    # Benchmark with hyperfine
    if command_exists hyperfine; then
        echo "⏱️  Running benchmark with hyperfine..."
        hyperfine \
            --runs $BENCHMARK_RUNS \
            --export-json "$PROFILE_DIR/threejs-benchmark.json" \
            --export-markdown "$PROFILE_DIR/threejs-benchmark.md" \
            --prepare 'rm -rf dist node_modules/.cache' \
            'rspack build --mode development'
    fi
    
    cd ../..
}

# Profile memory usage
profile_memory() {
    echo "🧠 Profiling memory usage..."
    
    cd "$EXAMPLES_DIR"
    
    # Use time command for basic memory stats
    echo "Running memory profiling..."
    /usr/bin/time -l rspack build --mode development 2>&1 | tee "$PROFILE_DIR/memory-usage.txt"
    
    # macOS: Use leaks command
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "Checking for memory leaks..."
        # Run build and capture PID for leak detection
        rspack build --mode development &
        BUILD_PID=$!
        sleep 5  # Let it run for a bit
        leaks $BUILD_PID > "$PROFILE_DIR/memory-leaks.txt" || true
        wait $BUILD_PID
    fi
    
    cd ../..
}

# Profile with perf (Linux only)
profile_with_perf() {
    if [[ "$OSTYPE" == "linux-gnu"* ]] && command_exists perf; then
        echo "🐧 Profiling with perf..."
        
        cd "$EXAMPLES_DIR"
        
        # CPU profiling
        perf record -g --call-graph=dwarf -o "$PROFILE_DIR/perf-cpu.data" \
            rspack build --mode development
        
        # Generate perf report
        perf report -i "$PROFILE_DIR/perf-cpu.data" > "$PROFILE_DIR/perf-report.txt"
        
        # Cache analysis
        perf stat -e cache-references,cache-misses,cycles,instructions \
            rspack build --mode development 2>&1 | tee "$PROFILE_DIR/perf-cache-stats.txt"
        
        cd ../..
    fi
}

# Profile with Instruments (macOS only)
profile_with_instruments() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "🍎 Setting up Instruments profiling..."
        
        cd "$EXAMPLES_DIR"
        
        echo "Starting Time Profiler..."
        # This will open Instruments GUI - user needs to manually start profiling
        xcrun xctrace record \
            --template "Time Profiler" \
            --output "$PROFILE_DIR/instruments-time.trace" \
            --launch -- rspack build --mode development || true
        
        cd ../..
    fi
}

# Analyze bundle composition
analyze_bundle() {
    echo "📊 Analyzing bundle composition..."
    
    cd "$EXAMPLES_DIR"
    
    # Build and capture stats
    rspack build --mode development --json > "$PROFILE_DIR/webpack-stats.json"
    
    # Basic analysis
    echo "Bundle analysis complete. Check webpack-stats.json for details."
    
    # Extract key metrics
    if command_exists jq; then
        echo "=== Bundle Analysis ===" > "$PROFILE_DIR/bundle-analysis.txt"
        echo "Build time: $(jq -r '.time' "$PROFILE_DIR/webpack-stats.json")ms" >> "$PROFILE_DIR/bundle-analysis.txt"
        echo "Asset count: $(jq -r '.assets | length' "$PROFILE_DIR/webpack-stats.json")" >> "$PROFILE_DIR/bundle-analysis.txt"
        echo "Module count: $(jq -r '.modules | length' "$PROFILE_DIR/webpack-stats.json")" >> "$PROFILE_DIR/bundle-analysis.txt"
        
        # Largest modules
        echo "=== Largest Modules ===" >> "$PROFILE_DIR/bundle-analysis.txt"
        jq -r '.modules | sort_by(.size) | reverse | .[0:10] | .[] | "\(.size) bytes - \(.name)"' \
            "$PROFILE_DIR/webpack-stats.json" >> "$PROFILE_DIR/bundle-analysis.txt"
    fi
    
    cd ../..
}

# Generate profiling report
generate_report() {
    echo "📝 Generating profiling report..."
    
    cat > "$PROFILE_DIR/README.md" << 'EOF'
# Rspack Performance Profiling Results

This directory contains profiling results for Rspack performance analysis.

## Files Generated

### CPU Profiling
- `threejs-flamegraph.svg` - Flamegraph showing CPU usage patterns
- `perf-report.txt` - Perf profiling report (Linux only)
- `instruments-time.trace` - Instruments trace file (macOS only)

### Memory Profiling  
- `memory-usage.txt` - Memory usage statistics
- `memory-leaks.txt` - Memory leak analysis (macOS only)

### Benchmarking
- `threejs-benchmark.json` - Hyperfine benchmark results
- `threejs-benchmark.md` - Benchmark results in markdown

### Bundle Analysis
- `webpack-stats.json` - Webpack stats JSON
- `bundle-analysis.txt` - Bundle composition analysis

## How to Analyze

### 1. CPU Profiling
Open `threejs-flamegraph.svg` in a browser to see where CPU time is spent.
Look for wide bars indicating hot functions.

### 2. Memory Analysis
Check `memory-usage.txt` for peak memory usage.
Review `memory-leaks.txt` for potential memory leaks.

### 3. Performance Comparison
Compare `threejs-benchmark.json` with baseline measurements.
Look for regressions in build time.

### 4. Bundle Impact
Review `bundle-analysis.txt` for module composition.
Check if bundle size has increased significantly.

## Next Steps

1. Identify hot functions from flamegraph
2. Look for memory allocation patterns
3. Compare with previous benchmark results
4. Focus optimization on the biggest bottlenecks
EOF

    echo "✅ Profiling complete! Results saved to: $PROFILE_DIR"
    echo ""
    echo "📋 Summary of generated files:"
    ls -la "$PROFILE_DIR"
    echo ""
    echo "🔍 To analyze results:"
    echo "  1. Open threejs-flamegraph.svg in browser for CPU analysis"
    echo "  2. Check threejs-benchmark.json for performance metrics"
    echo "  3. Review bundle-analysis.txt for bundle composition"
    echo "  4. See README.md for detailed analysis instructions"
}

# Main execution
main() {
    echo "Starting comprehensive Rspack profiling..."
    
    setup_profiling_tools
    build_with_profiling
    profile_threejs_benchmark
    profile_memory
    profile_with_perf
    analyze_bundle
    generate_report
    
    echo ""
    echo "🎉 Profiling complete!"
    echo "Results are available in: $PROFILE_DIR"
}

# Run main function
main "$@"