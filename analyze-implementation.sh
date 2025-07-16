#!/bin/bash
# cspell:ignore rspack shak oneline flamegraph hotspots

# Analyze the current swc-macro implementation for performance issues
set -e

echo "🔍 Analyzing SWC-Macro Implementation"
echo "====================================="

ANALYSIS_DIR="implementation-analysis"
mkdir -p "$ANALYSIS_DIR"

# Function to count lines and complexity
analyze_file() {
    local file="$1"
    local name="$2"
    
    if [ -f "$file" ]; then
        echo "=== $name ===" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        echo "File: $file" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        echo "Lines: $(wc -l < "$file")" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        echo "Size: $(wc -c < "$file") bytes" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        
        # Count loops and recursive calls
        echo "Loops (for/while): $(grep -c -E '(for |while )' "$file" || echo 0)" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        echo "Function calls: $(grep -c -E '\w+\(' "$file" || echo 0)" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        echo "HashMap operations: $(grep -c -E '(HashMap|HashSet|BTreeMap)' "$file" || echo 0)" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        echo "Clone operations: $(grep -c '\.clone()' "$file" || echo 0)" >> "$ANALYSIS_DIR/complexity-analysis.txt"
        echo "" >> "$ANALYSIS_DIR/complexity-analysis.txt"
    fi
}

echo "📊 Analyzing key performance-critical files..."

# Analyze runtime template (known bottleneck)
analyze_file "crates/rspack_core/src/dependency/runtime_template.rs" "Runtime Template"

# Analyze module concatenation plugin
analyze_file "crates/rspack_plugin_javascript/src/plugin/module_concatenation_plugin.rs" "Module Concatenation"

# Analyze memory GC
analyze_file "crates/rspack_core/src/utils/memory_gc.rs" "Memory GC"

# Analyze build chunk graph
analyze_file "crates/rspack_core/src/build_chunk_graph/new_code_splitter.rs" "Code Splitter"

# Look for SWC-specific optimizations
echo "🔍 Searching for SWC-macro specific code..."
find . -name "*.rs" -path "*/crates/*" -exec grep -l "swc_macro\|tree.*shak\|pure.*annotation" {} \; > "$ANALYSIS_DIR/swc-related-files.txt"

echo "SWC-related files found:" >> "$ANALYSIS_DIR/swc-analysis.txt"
cat "$ANALYSIS_DIR/swc-related-files.txt" >> "$ANALYSIS_DIR/swc-analysis.txt"
echo "" >> "$ANALYSIS_DIR/swc-analysis.txt"

# Analyze the shared module detection logic specifically
echo "🎯 Analyzing shared module detection (major bottleneck)..."
if [ -f "crates/rspack_core/src/dependency/runtime_template.rs" ]; then
    echo "=== Shared Module Detection Analysis ===" > "$ANALYSIS_DIR/shared-module-analysis.txt"
    
    # Extract the is_consume_shared_descendant function
    sed -n '/fn is_consume_shared_descendant/,/^}/p' \
        "crates/rspack_core/src/dependency/runtime_template.rs" \
        > "$ANALYSIS_DIR/shared-module-function.rs"
    
    echo "Function extracted to shared-module-function.rs" >> "$ANALYSIS_DIR/shared-module-analysis.txt"
    echo "Lines in function: $(wc -l < "$ANALYSIS_DIR/shared-module-function.rs")" >> "$ANALYSIS_DIR/shared-module-analysis.txt"
    
    # Count operations in the function
    echo "BFS operations: $(grep -c 'queue\|visited' "$ANALYSIS_DIR/shared-module-function.rs" || echo 0)" >> "$ANALYSIS_DIR/shared-module-analysis.txt"
    echo "HashMap operations: $(grep -c 'get_incoming_connections\|module_by_identifier' "$ANALYSIS_DIR/shared-module-function.rs" || echo 0)" >> "$ANALYSIS_DIR/shared-module-analysis.txt"
    echo "Module type checks: $(grep -c 'module_type()' "$ANALYSIS_DIR/shared-module-function.rs" || echo 0)" >> "$ANALYSIS_DIR/shared-module-analysis.txt"
fi

# Look for performance TODO comments
echo "📝 Finding performance TODOs..."
find crates -name "*.rs" -exec grep -n -H "TODO.*perf\|TODO.*performance\|TODO.*O(n\|FIXME.*slow\|FIXME.*performance" {} \; > "$ANALYSIS_DIR/performance-todos.txt"

# Count allocations and expensive operations
echo "💰 Counting expensive operations..."
echo "=== Expensive Operations Count ===" > "$ANALYSIS_DIR/expensive-ops.txt"

find crates -name "*.rs" -print0 | while IFS= read -r -d '' file; do
    if [ -f "$file" ]; then
        echo "File: $file" >> "$ANALYSIS_DIR/expensive-ops.txt"
        echo "  String allocations: $(grep -c 'String::new\|to_string\|format!' "$file" || echo 0)" >> "$ANALYSIS_DIR/expensive-ops.txt"
        echo "  Vec allocations: $(grep -c 'Vec::new\|vec!\|Vec::with_capacity' "$file" || echo 0)" >> "$ANALYSIS_DIR/expensive-ops.txt"
        echo "  HashMap allocations: $(grep -c 'HashMap::new\|HashSet::new' "$file" || echo 0)" >> "$ANALYSIS_DIR/expensive-ops.txt"
        echo "  Clone operations: $(grep -c '\.clone()' "$file" || echo 0)" >> "$ANALYSIS_DIR/expensive-ops.txt"
        echo "  Mutex locks: $(grep -c 'lock()\|write()\|read()' "$file" || echo 0)" >> "$ANALYSIS_DIR/expensive-ops.txt"
        echo "" >> "$ANALYSIS_DIR/expensive-ops.txt"
    fi
done

# Find the commit that introduced performance regression
echo "🕵️ Searching for recent changes..."
echo "=== Recent Changes in Performance-Critical Files ===" > "$ANALYSIS_DIR/recent-changes.txt"

# Check git log for runtime_template.rs
git log --oneline -n 10 -- crates/rspack_core/src/dependency/runtime_template.rs >> "$ANALYSIS_DIR/recent-changes.txt" || echo "No git history available"

echo "" >> "$ANALYSIS_DIR/recent-changes.txt"
echo "=== Recent Commits Mentioning Performance ===" >> "$ANALYSIS_DIR/recent-changes.txt"
git log --oneline --grep="perf\|performance\|slow\|speed\|optimization" -n 10 >> "$ANALYSIS_DIR/recent-changes.txt" || echo "No performance-related commits found"

# Generate summary report
echo "📋 Generating summary report..."
cat > "$ANALYSIS_DIR/README.md" << 'EOF'
# SWC-Macro Implementation Analysis

This analysis examines the current implementation for performance bottlenecks.

## Key Findings

### 1. Runtime Template Bottlenecks
- `runtime_template.rs` contains BFS traversal for shared module detection
- Called on every import statement generation
- No caching of traversal results

### 2. Performance TODOs
- Check `performance-todos.txt` for known optimization opportunities
- Module concatenation plugin has explicit O(n²) → O(n) TODO

### 3. Expensive Operations
- Review `expensive-ops.txt` for allocation patterns
- Look for high clone counts and mutex contention

### 4. Recent Changes
- `recent-changes.txt` shows commits that may have introduced regressions
- Focus on changes to runtime template and module processing

## Files Generated

- `complexity-analysis.txt` - Code complexity metrics
- `swc-analysis.txt` - SWC-specific code locations  
- `shared-module-analysis.txt` - Analysis of BFS bottleneck
- `shared-module-function.rs` - Extracted bottleneck function
- `performance-todos.txt` - Known performance issues
- `expensive-ops.txt` - Operation cost analysis
- `recent-changes.txt` - Recent changes in critical files

## Optimization Priorities

1. **Cache shared module detection results**
2. **Implement O(n) module concatenation algorithm**  
3. **Reduce string allocations in hot paths**
4. **Optimize mutex usage in memory GC**

## Next Steps

1. Profile with flamegraph to confirm bottlenecks
2. Implement caching for shared module detection
3. Optimize allocation patterns in hot paths
4. Consider parallelization for independent operations
EOF

echo ""
echo "✅ Implementation analysis complete!"
echo ""
echo "📁 Results saved to implementation-analysis/:"
ls -la "$ANALYSIS_DIR"
echo ""
echo "🔍 Key insights:"
echo "  📊 Check complexity-analysis.txt for code metrics"
echo "  🎯 Review shared-module-analysis.txt for BFS bottleneck"
echo "  📝 See performance-todos.txt for known issues"
echo "  💰 Check expensive-ops.txt for allocation hotspots"