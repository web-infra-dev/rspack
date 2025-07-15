#!/bin/bash

echo "🔍 Basic Example - ThreeJS-Level Performance Benchmark"
echo "====================================================="

cd examples/basic

# Create additional complexity to match threejs levels
echo "📦 Scaling up basic example complexity..."

# Create more shared modules to stress test is_consume_shared_descendant
mkdir -p test-modules/shared

# Generate 50 shared utility modules
for i in {1..50}; do
    cat > "test-modules/shared/utility-$i.js" << EOF
// Shared utility module $i
export const utility$i = {
    process() {
        return 'utility-$i-processed';
    },
    transform(data) {
        return data.map(x => x + $i);
    },
    config: {
        id: $i,
        name: 'utility-$i'
    }
};

export default utility$i;
EOF
done

# Create 20 feature modules that import shared utilities
mkdir -p test-modules/features
for i in {1..20}; do
    # Each feature imports 5-10 random shared utilities
    imports=""
    for j in {1..7}; do
        util_num=$(( (i * j) % 50 + 1 ))
        imports="$imports
import { utility$util_num } from '../shared/utility-$util_num.js';"
    done
    
    cat > "test-modules/features/feature-$i.js" << EOF
// Feature module $i with multiple shared dependencies
$imports

export class Feature$i {
    constructor() {
        this.id = $i;
        this.utilities = [$(for j in {1..7}; do echo -n "utility\$(( (${j} * ${i}) % 50 + 1 ))"; [ $j -lt 7 ] && echo -n ", "; done)];
    }
    
    execute() {
        return this.utilities.map(u => u.process()).join('-');
    }
}

export default Feature$i;
EOF
done

# Update main entry to import all features
cat >> index.js << 'EOF'

// Import all test features to create complex dependency graph
import Feature1 from './test-modules/features/feature-1.js';
import Feature2 from './test-modules/features/feature-2.js';
import Feature3 from './test-modules/features/feature-3.js';
import Feature4 from './test-modules/features/feature-4.js';
import Feature5 from './test-modules/features/feature-5.js';
import Feature6 from './test-modules/features/feature-6.js';
import Feature7 from './test-modules/features/feature-7.js';
import Feature8 from './test-modules/features/feature-8.js';
import Feature9 from './test-modules/features/feature-9.js';
import Feature10 from './test-modules/features/feature-10.js';

// Use features to ensure they're not tree-shaken
const features = [Feature1, Feature2, Feature3, Feature4, Feature5, 
                 Feature6, Feature7, Feature8, Feature9, Feature10];

console.log('Loaded features:', features.length);
EOF

# Create enhanced config with more sharing
cat > rspack.config.enhanced-sharing.cjs << 'EOF'
const rspack = require("@rspack/core");

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
		mergeDuplicateChunks: true
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "enhanced_basic_example", 
			filename: "remoteEntry.js",

			exposes: {
				"./cjs-test": "./cjs-modules/legacy-utils.js",
				"./cjs-data-processor": "./cjs-modules/data-processor.js",
				"./cjs-pure-helper": "./cjs-modules/pure-cjs-helper.js",
				"./cjs-module-exports": "./cjs-modules/module-exports-pattern.js"
			},

			shared: {
				// Share all the generated utilities (this will stress test is_consume_shared_descendant)
				...Array.from({length: 50}, (_, i) => i + 1).reduce((acc, i) => {
					acc[`./test-modules/shared/utility-${i}.js`] = {
						singleton: true,
						eager: false,
						requiredVersion: false,
						shareKey: `utility-${i}`
					};
					return acc;
				}, {}),
				
				// Share all features
				...Array.from({length: 20}, (_, i) => i + 1).reduce((acc, i) => {
					acc[`./test-modules/features/feature-${i}.js`] = {
						singleton: true,
						eager: false,
						requiredVersion: false,
						shareKey: `feature-${i}`
					};
					return acc;
				}, {}),

				// Original shared modules
				"./shared/utils": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "utility-lib"
				},
				"./shared/components": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "component-lib"
				},
				"./shared/api": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "api-lib"
				},

				// External libs
				"lodash-es": {
					singleton: true,
					requiredVersion: "^4.17.21",
					eager: false,
					shareKey: "lodash-es",
					shareScope: "default"
				}
			},

			remotes: {
				remote_app: "remote_app@http://localhost:3001/remoteEntry.js",
				cjs_test_remote: "cjs_test@http://localhost:3002/remoteEntry.js"
			}
		})
	]
};
EOF

# Create baseline config (no sharing)
cat > rspack.config.baseline.cjs << 'EOF'
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
		innerGraph: true
	}
	// NO MODULE FEDERATION
};
EOF

echo ""
echo "🏃 Running ThreeJS-level complexity benchmark..."

# Function to run benchmark
run_complex_benchmark() {
    local name=$1
    local config=$2
    local iterations=3
    local times=()
    
    echo "" >&2
    echo "🎯 $name ($iterations iterations)..." >&2
    
    for i in $(seq 1 $iterations); do
        rm -rf dist
        start=$(date +%s%N)
        node ../../packages/rspack-cli/bin/rspack.js build --config $config > /dev/null 2>&1
        end=$(date +%s%N)
        duration=$(( (end - start) / 1000000 ))
        times+=($duration)
        echo "  Run $i: ${duration}ms" >&2
    done
    
    # Calculate stats
    sum=0
    min=${times[0]}
    max=${times[0]}
    for time in "${times[@]}"; do
        sum=$((sum + time))
        if [ $time -lt $min ]; then min=$time; fi
        if [ $time -gt $max ]; then max=$time; fi
    done
    avg=$((sum / iterations))
    
    echo "  Average: ${avg}ms (min: ${min}ms, max: ${max}ms)" >&2
    echo $avg
}

# Run benchmarks
baseline_time=$(run_complex_benchmark "Baseline (No Sharing)" "rspack.config.baseline.cjs")
original_time=$(run_complex_benchmark "Original Module Federation" "rspack.config.cjs") 
enhanced_time=$(run_complex_benchmark "Enhanced Sharing (70 shared modules)" "rspack.config.enhanced-sharing.cjs")

# Calculate impacts
original_impact=$(( (original_time - baseline_time) * 100 / baseline_time ))
enhanced_impact=$(( (enhanced_time - baseline_time) * 100 / baseline_time ))

echo ""
echo "📊 ThreeJS-Level Performance Analysis:"
echo "======================================"
echo "  Baseline (no sharing):     ${baseline_time}ms"
echo "  Original Module Federation: ${original_time}ms (+${original_impact}%)"
echo "  Enhanced sharing (70 modules): ${enhanced_time}ms (+${enhanced_impact}%)"
echo ""

if [ $enhanced_impact -gt 50 ]; then
    echo "🚨 SEVERE PERFORMANCE REGRESSION with enhanced sharing: +${enhanced_impact}%"
    echo "   This demonstrates the is_consume_shared_descendant bottleneck at scale!"
elif [ $enhanced_impact -gt 20 ]; then
    echo "⚠️  SIGNIFICANT performance impact with enhanced sharing: +${enhanced_impact}%"
else
    echo "✅ Reasonable performance impact: +${enhanced_impact}%"
fi

# Get detailed stats for the enhanced config
echo ""
echo "📈 Enhanced Config Statistics:"
rm -rf dist
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.enhanced-sharing.cjs --json > enhanced-stats.json 2>/dev/null

if [ -f "enhanced-stats.json" ]; then
    echo "  Modules: $(jq -r '.modules | length // "N/A"' enhanced-stats.json)"
    echo "  Chunks: $(jq -r '.chunks | length // "N/A"' enhanced-stats.json)"
    echo "  Build time: $(jq -r '.time // "N/A"' enhanced-stats.json)ms"
fi

# Clean up temp files
rm -f rspack.config.enhanced-sharing.cjs rspack.config.baseline.cjs enhanced-stats.json

cd ../..

echo ""
echo "✅ ThreeJS-level benchmark complete!"
echo ""
echo "💡 This test created 70 shared modules (50 utilities + 20 features)"
echo "   to stress test the is_consume_shared_descendant function at scale."
echo ""
echo "🎯 Expected behavior:"
echo "   - Enhanced sharing should show significant regression"
echo "   - Performance should scale poorly with number of shared modules"
echo "   - This confirms the need for BuildMeta caching optimization"