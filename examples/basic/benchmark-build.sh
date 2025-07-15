#!/bin/bash

echo "🔍 Rspack Basic Example Performance Benchmark"
echo "============================================"

# Skip build - using locally compiled version
echo "📦 Using locally compiled Rspack..."

# Clean previous builds
rm -rf dist

# Function to run benchmark
run_benchmark() {
    local name=$1
    local iterations=5
    local times=()
    
    echo ""
    echo "🏃 Running $name benchmark ($iterations iterations)..."
    
    for i in $(seq 1 $iterations); do
        # Clean dist before each run
        rm -rf dist
        
        # Time the build
        start=$(date +%s%N)
        node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs > /dev/null 2>&1
        end=$(date +%s%N)
        
        # Calculate duration in milliseconds
        duration=$(( (end - start) / 1000000 ))
        times+=($duration)
        echo "  Run $i: ${duration}ms"
    done
    
    # Calculate average
    sum=0
    for time in "${times[@]}"; do
        sum=$((sum + time))
    done
    avg=$((sum / iterations))
    
    echo "  Average: ${avg}ms"
    echo ""
}

# Run benchmarks
echo "🔨 Starting benchmark suite..."

# Standard build
run_benchmark "Standard Build"

# Check output size
if [ -d "dist" ]; then
    echo "📊 Build Output Stats:"
    echo "  Total size: $(du -sh dist | cut -f1)"
    echo "  File count: $(find dist -type f | wc -l)"
    echo "  Main bundle: $(ls -lh dist/main.js 2>/dev/null | awk '{print $5}')"
fi

echo ""
echo "✅ Benchmark complete!"