#!/bin/bash

echo "🔍 Rspack Module Federation Sharing Performance Benchmark"
echo "========================================================"

# Skip build - already built
echo "📦 Using pre-built Rspack release binary..."

# Clean previous builds
rm -rf dist

# Function to run benchmark
run_benchmark() {
    local name=$1
    local config=$2
    local iterations=5
    local times=()
    
    echo ""
    echo "🏃 Running $name benchmark ($iterations iterations)..."
    
    for i in $(seq 1 $iterations); do
        # Clean dist before each run
        rm -rf dist
        
        # Time the build
        start=$(date +%s%N)
        node ../../packages/rspack-cli/bin/rspack.js build --config $config > /dev/null 2>&1
        end=$(date +%s%N)
        
        # Calculate duration in milliseconds
        duration=$(( (end - start) / 1000000 ))
        times+=($duration)
        echo "  Run $i: ${duration}ms"
    done
    
    # Calculate average and min/max
    sum=0
    min=${times[0]}
    max=${times[0]}
    
    for time in "${times[@]}"; do
        sum=$((sum + time))
        if [ $time -lt $min ]; then min=$time; fi
        if [ $time -gt $max ]; then max=$time; fi
    done
    
    avg=$((sum / iterations))
    
    echo "  Average: ${avg}ms (min: ${min}ms, max: ${max}ms)"
    
    # Store average in global variable instead of echo
    LAST_AVG=$avg
}

# Create config without sharing
cat > rspack.config.no-sharing.cjs << 'EOF'
const rspack = require("@rspack/core");

/** @type {import('@rspack/cli').Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	mode: "development",
	devtool: false,
	output: {
		clean: true
	},
	resolve: {
		alias: {
			"@cjs-test": require
				.resolve("./cjs-modules/package.json")
				.replace("/package.json", ""),
			"cjs-modules": require
				.resolve("./cjs-modules/package.json")
				.replace("/package.json", "")
		}
	},
	optimization: {
		minimize: false,
		usedExports: true,
		providedExports: true,
		sideEffects: false,
		concatenateModules: false,
		innerGraph: true,
		mangleExports: true,
		removeAvailableModules: true,
		removeEmptyChunks: true,
		mergeDuplicateChunks: true,
		moduleIds: "named",
		chunkIds: "named",
		realContentHash: true
	}
	// NO MODULE FEDERATION PLUGIN
};
EOF

echo "🔨 Starting benchmark comparison..."

# Run benchmarks
echo ""
echo "1️⃣ WITHOUT Module Federation Sharing:"
run_benchmark "No Sharing" "rspack.config.no-sharing.cjs"
time_without=$LAST_AVG

echo ""
echo "2️⃣ WITH Module Federation Sharing (Full Config):"
run_benchmark "With Sharing" "rspack.config.cjs"
time_with=$LAST_AVG

# Calculate performance impact
if [ "$time_without" -gt 0 ]; then
    impact=$(( (time_with - time_without) * 100 / time_without ))
    echo ""
    echo "📊 Performance Impact Analysis:"
    echo "================================"
    echo "  Without sharing: ${time_without}ms"
    echo "  With sharing:    ${time_with}ms"
    echo "  Difference:      $((time_with - time_without))ms"
    echo "  Performance impact: ${impact}%"
    
    if [ $impact -gt 0 ]; then
        echo "  ⚠️  Module Federation sharing adds ${impact}% to build time"
    else
        echo "  ✅ Module Federation sharing has minimal impact"
    fi
fi

# Check output differences
echo ""
echo "📦 Build Output Comparison:"
echo "================================"

# Build both configs for size comparison
rm -rf dist
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.no-sharing.cjs > /dev/null 2>&1
no_sharing_size=$(du -sk dist | cut -f1)
no_sharing_files=$(find dist -type f | wc -l)

rm -rf dist
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs > /dev/null 2>&1
with_sharing_size=$(du -sk dist | cut -f1)
with_sharing_files=$(find dist -type f | wc -l)

echo "  Without sharing: ${no_sharing_size}KB (${no_sharing_files} files)"
echo "  With sharing:    ${with_sharing_size}KB (${with_sharing_files} files)"

# Clean up
rm -f rspack.config.no-sharing.cjs

echo ""
echo "✅ Benchmark complete!"