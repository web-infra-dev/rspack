#!/bin/bash
# cspell:ignore rspack dtrace execname ustack

echo "🔍 Detailed Function-Level Performance Analysis"
echo "=============================================="

cd examples/basic

# Create a detailed timing build that outputs more verbose information
echo "🎯 Running detailed timing analysis..."

# Build with timing information
echo "=== DETAILED TIMING ANALYSIS ===" > ../../profiling-results/detailed-timing.txt
echo "Date: $(date)" >> ../../profiling-results/detailed-timing.txt
echo "" >> ../../profiling-results/detailed-timing.txt

echo "Running build with Module Federation and capturing detailed output..."

# Run with more verbose output to see timing breakdowns
rm -rf dist
RUST_LOG=debug node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs 2>&1 | \
    grep -E "(took|elapsed|duration|ms|seconds)" | \
    head -50 >> ../../profiling-results/detailed-timing.txt

echo "" >> ../../profiling-results/detailed-timing.txt
echo "=== BUILD STATS WITH SHARING ===" >> ../../profiling-results/detailed-timing.txt

# Get detailed stats output
rm -rf dist
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs --json > ../../profiling-results/build-stats-with-sharing.json 2>/dev/null

# Extract timing information from stats
if [ -f "../../profiling-results/build-stats-with-sharing.json" ]; then
    echo "Build time: $(jq -r '.time // "N/A"' ../../profiling-results/build-stats-with-sharing.json)ms" >> ../../profiling-results/detailed-timing.txt
    echo "Module count: $(jq -r '.modules | length // "N/A"' ../../profiling-results/build-stats-with-sharing.json)" >> ../../profiling-results/detailed-timing.txt
    echo "Chunk count: $(jq -r '.chunks | length // "N/A"' ../../profiling-results/build-stats-with-sharing.json)" >> ../../profiling-results/detailed-timing.txt
fi

echo "" >> ../../profiling-results/detailed-timing.txt
echo "=== BUILD STATS WITHOUT SHARING ===" >> ../../profiling-results/detailed-timing.txt

# Create simple config and get stats
cat > rspack.config.no-sharing-temp.cjs << 'EOF'
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

rm -rf dist
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.no-sharing-temp.cjs --json > ../../profiling-results/build-stats-without-sharing.json 2>/dev/null

if [ -f "../../profiling-results/build-stats-without-sharing.json" ]; then
    echo "Build time: $(jq -r '.time // "N/A"' ../../profiling-results/build-stats-without-sharing.json)ms" >> ../../profiling-results/detailed-timing.txt
    echo "Module count: $(jq -r '.modules | length // "N/A"' ../../profiling-results/build-stats-without-sharing.json)" >> ../../profiling-results/detailed-timing.txt
    echo "Chunk count: $(jq -r '.chunks | length // "N/A"' ../../profiling-results/build-stats-without-sharing.json)" >> ../../profiling-results/detailed-timing.txt
fi

rm -f rspack.config.no-sharing-temp.cjs

echo "" >> ../../profiling-results/detailed-timing.txt
echo "=== FUNCTION CALL ANALYSIS ===" >> ../../profiling-results/detailed-timing.txt

# Try to use time and profile with dtrace on macOS if available
if command -v dtrace >/dev/null 2>&1; then
    echo "Running dtrace profiling for function calls..." >> ../../profiling-results/detailed-timing.txt
    
    # Simple dtrace script to track function calls
    sudo dtrace -n 'profile-997 /execname == "node"/ { @[ustack()] = count(); }' -o ../../profiling-results/dtrace-profile.txt &
    DTRACE_PID=$!
    
    # Run build
    rm -rf dist
    node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.cjs > /dev/null 2>&1
    
    # Stop dtrace
    sleep 1
    sudo kill $DTRACE_PID 2>/dev/null || true
    
    echo "Dtrace output saved to dtrace-profile.txt" >> ../../profiling-results/detailed-timing.txt
else
    echo "Dtrace not available, skipping detailed function profiling" >> ../../profiling-results/detailed-timing.txt
fi

cd ../..

echo ""
echo "✅ Detailed profiling analysis complete!"
echo ""
echo "📊 Results in profiling-results/:"
echo "  📋 detailed-timing.txt - Detailed timing breakdown"
echo "  📊 build-stats-with-sharing.json - Build statistics WITH sharing"
echo "  📊 build-stats-without-sharing.json - Build statistics WITHOUT sharing"
if [ -f "profiling-results/dtrace-profile.txt" ]; then
    echo "  🔍 dtrace-profile.txt - Function call profiling"
fi

echo ""
echo "🔍 To find next highest costing functions:"
echo "  1. Check detailed-timing.txt for explicit timing information"
echo "  2. Compare module/chunk counts between builds"
echo "  3. Look for patterns in the build output"