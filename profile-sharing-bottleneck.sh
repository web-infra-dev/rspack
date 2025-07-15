#!/bin/bash

echo "🔍 Module Federation Sharing Performance Profiling"
echo "================================================="

# Check if flamegraph is available
if ! command -v flamegraph >/dev/null 2>&1; then
    echo "📦 Installing flamegraph..."
    cargo install flamegraph
fi

cd examples/basic

# Clean any existing profiles
rm -rf ../../profiling-results
mkdir -p ../../profiling-results

echo "🎯 Profiling WITH Module Federation sharing (current bottleneck)..."

# Clean build
rm -rf dist

# Profile the build that has Module Federation sharing enabled
flamegraph --freq 997 --min-width 0.01 \
    -o ../../profiling-results/with-sharing-flamegraph.svg -- \
    node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs

echo ""
echo "🎯 Profiling WITHOUT Module Federation sharing (baseline)..."

# Clean build  
rm -rf dist

# Create simple config without sharing for comparison
cat > rspack.config.no-sharing-profile.cjs << 'EOF'
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
	// NO MODULE FEDERATION - baseline performance
};
EOF

# Profile the baseline build
flamegraph --freq 997 --min-width 0.01 \
    -o ../../profiling-results/without-sharing-flamegraph.svg -- \
    node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.no-sharing-profile.cjs

# Clean up temp config
rm -f rspack.config.no-sharing-profile.cjs

echo ""
echo "🔍 Generating performance comparison..."

# Time both builds for comparison
echo "=== Performance Comparison ===" > ../../profiling-results/performance-report.txt
echo "Date: $(date)" >> ../../profiling-results/performance-report.txt
echo "" >> ../../profiling-results/performance-report.txt

echo "Testing WITH Module Federation sharing..." >> ../../profiling-results/performance-report.txt
for i in {1..3}; do
    rm -rf dist
    start=$(date +%s%N)
    node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs > /dev/null 2>&1
    end=$(date +%s%N)
    duration=$(( (end - start) / 1000000 ))
    echo "  Run $i: ${duration}ms" >> ../../profiling-results/performance-report.txt
done

echo "" >> ../../profiling-results/performance-report.txt
echo "Testing WITHOUT Module Federation sharing..." >> ../../profiling-results/performance-report.txt

cat > rspack.config.no-sharing-profile.cjs << 'EOF'
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
};
EOF

for i in {1..3}; do
    rm -rf dist
    start=$(date +%s%N)
    node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.no-sharing-profile.cjs > /dev/null 2>&1
    end=$(date +%s%N)
    duration=$(( (end - start) / 1000000 ))
    echo "  Run $i: ${duration}ms" >> ../../profiling-results/performance-report.txt
done

rm -f rspack.config.no-sharing-profile.cjs

echo "" >> ../../profiling-results/performance-report.txt
echo "=== Key Functions to Analyze ===" >> ../../profiling-results/performance-report.txt
echo "1. is_consume_shared_descendant - BFS traversal bottleneck" >> ../../profiling-results/performance-report.txt
echo "2. Module Federation plugin initialization" >> ../../profiling-results/performance-report.txt
echo "3. Shared module dependency resolution" >> ../../profiling-results/performance-report.txt
echo "4. Runtime template generation" >> ../../profiling-results/performance-report.txt

cd ../..

echo ""
echo "✅ Profiling complete!"
echo ""
echo "📊 Results:"
echo "  📈 with-sharing-flamegraph.svg - Flame graph WITH Module Federation"
echo "  📈 without-sharing-flamegraph.svg - Flame graph WITHOUT Module Federation"  
echo "  📋 performance-report.txt - Timing comparison"
echo ""
echo "🔍 To analyze:"
echo "  1. Open the SVG files in a browser"
echo "  2. Look for 'is_consume_shared_descendant' function in the flame graph"
echo "  3. Compare the width of function calls between the two graphs"
echo "  4. Focus on runtime_template.rs and module federation related functions"
echo ""
echo "💡 Expected findings:"
echo "  - is_consume_shared_descendant should show up prominently in WITH sharing"
echo "  - BFS traversal should be visible as wide bars in the flame graph"
echo "  - Module graph operations should dominate the profile"