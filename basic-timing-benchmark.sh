#!/bin/bash
# cspell:ignore rspack

echo "🔍 Basic Example Timing Benchmark (CI-style)"
echo "============================================"

cd examples/basic

# Clean up any test modules from previous runs
rm -rf test-modules

# Function to run timing benchmark (similar to CI)
run_timing_benchmark() {
    local name=$1
    local config=$2
    local iterations=5
    local times=()
    
    echo ""
    echo "📊 Running $name benchmark ($iterations iterations)..."
    
    for i in $(seq 1 $iterations); do
        # Clean dist before each run
        rm -rf dist
        
        # Time the build (same as CI methodology)
        start=$(date +%s%N)
        node ../../packages/rspack-cli/bin/rspack.js build --config $config > /dev/null 2>&1
        end=$(date +%s%N)
        
        # Calculate duration in milliseconds
        duration=$(( (end - start) / 1000000 ))
        times+=($duration)
        echo "  Run $i: ${duration}ms"
    done
    
    # Calculate statistics (same as CI)
    sum=0
    min=${times[0]}
    max=${times[0]}
    
    for time in "${times[@]}"; do
        sum=$((sum + time))
        if [ $time -lt $min ]; then min=$time; fi
        if [ $time -gt $max ]; then max=$time; fi
    done
    
    avg=$((sum / iterations))
    
    # Calculate standard deviation (simplified)
    variance=0
    for time in "${times[@]}"; do
        diff=$((time - avg))
        variance=$((variance + diff * diff))
    done
    variance=$((variance / iterations))
    stddev=$(echo "sqrt($variance)" | bc -l 2>/dev/null | cut -d. -f1 2>/dev/null || echo "N/A")
    
    echo "  │ Average: ${avg}ms │ Min: ${min}ms │ Max: ${max}ms │ StdDev: ${stddev}ms │"
    echo "  └─────────────────────────────────────────────────────────────────────┘"
    
    # Store result in global variable instead of return code
    LAST_RESULT=$avg
}

echo "🎯 Testing basic example performance (current configuration)..."

# Test 1: Current Module Federation config
run_timing_benchmark "Basic w/ Module Federation" "rspack.config.cjs"
mf_time=$LAST_RESULT

# Test 2: Simple config without Module Federation
cat > rspack.config.simple.cjs << 'EOF'
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

run_timing_benchmark "Basic w/o Module Federation" "rspack.config.simple.cjs"
simple_time=$LAST_RESULT

# Test 3: Production mode
cat > rspack.config.production.cjs << 'EOF'
const rspack = require("@rspack/core");

module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	mode: "production",
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
		minimize: false, // Keep false to focus on compilation time, not minification
		usedExports: true,
		providedExports: true,
		sideEffects: false,
		concatenateModules: true,
		innerGraph: true,
		mangleExports: true,
		removeAvailableModules: true,
		removeEmptyChunks: true,
		mergeDuplicateChunks: true
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "basic_example_prod",
			filename: "remoteEntry.js",
			exposes: {
				"./cjs-test": "./cjs-modules/legacy-utils.js",
				"./cjs-data-processor": "./cjs-modules/data-processor.js",
			},
			shared: {
				"./shared/utils": {
					singleton: true,
					eager: false,
					shareKey: "utility-lib"
				},
				"./shared/components": {
					singleton: true,
					eager: false,
					shareKey: "component-lib"
				},
				"./shared/api": {
					singleton: true,
					eager: false,
					shareKey: "api-lib"
				},
				"lodash-es": {
					singleton: true,
					requiredVersion: "^4.17.21",
					eager: false,
					shareKey: "lodash-es"
				}
			}
		})
	]
};
EOF

run_timing_benchmark "Basic Production w/ MF" "rspack.config.production.cjs"
prod_time=$LAST_RESULT

# Calculate performance impacts
if [ $simple_time -gt 0 ]; then
    mf_impact=$(( (mf_time - simple_time) * 100 / simple_time ))
    prod_impact=$(( (prod_time - simple_time) * 100 / simple_time ))
else
    mf_impact=0
    prod_impact=0
fi

# Generate CI-style report
echo ""
echo "📋 Basic Example Performance Report (CI-style)"
echo "=============================================="
echo "| Configuration           | Time    | vs Baseline | Status |"
echo "|-------------------------|---------|-------------|--------|"
printf "| Simple (baseline)       | %4dms   | -           | ✅     |\n" $simple_time
printf "| Development + MF        | %4dms   | +%2d%%        | " $mf_time $mf_impact
if [ $mf_impact -gt 20 ]; then
    echo "🚨     |"
elif [ $mf_impact -gt 10 ]; then
    echo "⚠️      |"
else
    echo "✅     |"
fi
printf "| Production + MF         | %4dms   | +%2d%%        | " $prod_time $prod_impact
if [ $prod_impact -gt 20 ]; then
    echo "🚨     |"
elif [ $prod_impact -gt 10 ]; then
    echo "⚠️      |"
else
    echo "✅     |"
fi

echo ""
echo "🎯 Performance Analysis:"
echo "  • Module Federation overhead: +${mf_impact}% in development"
echo "  • Production mode overhead: +${prod_impact}% vs baseline"

if [ $mf_impact -gt 15 ]; then
    echo "  🚨 PERFORMANCE REGRESSION: Module Federation adds significant overhead"
    echo "     This confirms the is_consume_shared_descendant bottleneck"
elif [ $mf_impact -gt 5 ]; then
    echo "  ⚠️  Moderate performance impact from Module Federation"
else
    echo "  ✅ Acceptable Module Federation overhead"
fi

# Get module counts for analysis
echo ""
echo "📊 Build Statistics:"

# MF stats
rm -rf dist
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs --json > mf-stats.json 2>/dev/null
if [ -f "mf-stats.json" ]; then
    mf_modules=$(jq -r '.modules | length // 0' mf-stats.json 2>/dev/null || echo "0")
    mf_chunks=$(jq -r '.chunks | length // 0' mf-stats.json 2>/dev/null || echo "0")
    echo "  • Module Federation: ${mf_modules} modules, ${mf_chunks} chunks"
fi

# Simple stats  
rm -rf dist
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.simple.cjs --json > simple-stats.json 2>/dev/null
if [ -f "simple-stats.json" ]; then
    simple_modules=$(jq -r '.modules | length // 0' simple-stats.json 2>/dev/null || echo "0")
    simple_chunks=$(jq -r '.chunks | length // 0' simple-stats.json 2>/dev/null || echo "0")
    echo "  • Simple config: ${simple_modules} modules, ${simple_chunks} chunks"
fi

# Clean up
rm -f rspack.config.simple.cjs rspack.config.production.cjs
rm -f mf-stats.json simple-stats.json

cd ../..

echo ""
echo "✅ Basic example timing benchmark complete!"
echo ""
echo "💡 This provides CI-style timing data for the basic example"
echo "   comparable to the threejs benchmark methodology."