# Incremental Module Federation Performance Optimization Plan

## Overview
This plan implements the unified BuildMeta optimization from `IMPLEMENTATION_PLAN.md` in safe, testable increments to avoid breaking changes while delivering immediate performance benefits.

## Phase 1: ESM Import Optimization (Primary Bottleneck)
**Goal**: Fix the 21.61% performance regression in ESM imports
**Risk**: Low - only touches ESM runtime template
**Expected Impact**: Eliminate primary bottleneck, ~50% of total performance gains

### Step 1.1: Add BuildMeta Fields (Infrastructure)
**Files**: `crates/rspack_core/src/build_meta.rs`
- Add `is_shared_descendant: Option<bool>` field
- Add `effective_shared_key: Option<String>` field  
- Ensure serialization works correctly
- **Test**: Unit test BuildMeta serialization/deserialization

### Step 1.2: Add ESM-Only Shared Detection Function
**Files**: `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`
- Add `mark_esm_shared_descendants()` function (simplified version)
- Only process ESM imports initially
- Set BuildMeta fields for shared ESM modules
- **Test**: Unit test that BuildMeta fields are set correctly for ESM shared modules

### Step 1.3: Replace ESM BFS with BuildMeta Lookup
**Files**: `crates/rspack_core/src/dependency/runtime_template.rs`
- Modify `is_consume_shared_descendant()` to check BuildMeta first
- Keep original BFS as fallback for safety
- Add feature flag or environment variable to control behavior
- **Test**: Integration test that ESM imports still work correctly
- **Test**: Performance test showing improvement in ESM import speed

### Step 1.4: ESM-Only Performance Validation
- Run existing Module Federation test suite
- Run threejs benchmark to verify 21.61% regression is eliminated
- Ensure no CommonJS functionality is affected
- **Rollback plan**: Remove BuildMeta fields if any issues

## Phase 2: CommonJS Exports Optimization
**Goal**: Optimize `should_apply_to_module()` function
**Risk**: Medium - affects CommonJS tree-shaking
**Expected Impact**: ~30% additional performance gains

### Step 2.1: Extend Shared Detection to CommonJS Exports
**Files**: `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`
- Extend `mark_esm_shared_descendants()` to include CommonJS exports
- Ensure BuildMeta is set for CommonJS modules with shared context
- **Test**: Unit test CommonJS modules get correct BuildMeta

### Step 2.2: Optimize ConsumeSharedExportsDependency Gradually
**Files**: `crates/rspack_plugin_javascript/src/dependency/commonjs/consume_shared_exports_dependency.rs`
- Add BuildMeta check as first option in `should_apply_to_module()`
- Keep existing logic as fallback
- Gradually reduce reliance on expensive string matching
- **Test**: CommonJS exports tree-shaking still works correctly

### Step 2.3: CommonJS Exports Performance Validation
- Run CommonJS-specific test suite
- Verify tree-shaking annotations are still generated correctly
- Measure performance improvement in CommonJS-heavy builds
- **Rollback plan**: Revert to original `should_apply_to_module()` logic

## Phase 3: CommonJS Requires Optimization
**Goal**: Optimize `detect_consume_shared_context()` function
**Risk**: Medium - affects CommonJS require handling
**Expected Impact**: ~15% additional performance gains

### Step 3.1: Extend Shared Detection to CommonJS Requires
**Files**: `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`
- Include CommonJS require dependencies in shared detection
- Ensure require() calls get proper BuildMeta context
- **Test**: CommonJS require() calls maintain shared context

### Step 3.2: Optimize CommonJS Require Detection
**Files**: `crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_require_dependency.rs`
- Modify `detect_consume_shared_context()` to check BuildMeta first
- Keep module graph traversal as fallback
- **Test**: CommonJS require() tree-shaking works correctly

## Phase 4: Complete CommonJS Optimization
**Goal**: Finish all CommonJS optimizations
**Risk**: Low - incremental improvements on proven base
**Expected Impact**: Remaining performance gains

### Step 4.1: Optimize CommonJS Exports Dependency
**Files**: `crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_exports_dependency.rs`
- Replace shared key detection logic with direct BuildMeta access
- **Test**: All CommonJS export patterns work correctly

### Step 4.2: Remove Fallback Logic (Final Cleanup)
- Remove expensive fallback logic from all optimized functions
- Clean up feature flags and environment variables
- **Test**: Full regression test suite

## Phase 5: Performance Validation & Monitoring
**Goal**: Validate complete system performance
**Risk**: None - validation only
**Expected Impact**: Confirm 44% Module Federation overhead elimination

### Step 5.1: Comprehensive Performance Testing
- Run full threejs benchmark suite
- Run basic example timing benchmarks
- Test with enhanced sharing (70+ shared modules)
- Verify linear scaling with module count

### Step 5.2: Production Readiness
- Stress test with large applications
- Memory usage validation
- Performance regression monitoring setup

## Safety Measures Throughout Implementation

### 1. Feature Flags
```rust
// Environment variable to control optimization behavior
const USE_BUILDMETA_OPTIMIZATION: bool = std::env::var("RSPACK_BUILDMETA_OPT")
    .map(|v| v == "1" || v == "true")
    .unwrap_or(false);
```

### 2. Gradual Fallback Strategy
```rust
fn is_consume_shared_descendant_optimized(module_graph: &ModuleGraph, module_id: &ModuleIdentifier) -> bool {
    // Try BuildMeta first (fast path)
    if let Some(module) = module_graph.module_by_identifier(module_id) {
        if let Some(is_descendant) = module.build_meta().is_shared_descendant {
            return is_descendant;
        }
    }
    
    // Fallback to original BFS (for safety)
    if USE_BUILDMETA_OPTIMIZATION {
        // Log warning about missing BuildMeta
        eprintln!("Warning: Missing BuildMeta for module {}, falling back to BFS", module_id);
    }
    is_consume_shared_descendant_original(module_graph, module_id)
}
```

### 3. Comprehensive Testing Strategy
- **Unit tests**: Each BuildMeta field modification
- **Integration tests**: End-to-end Module Federation scenarios
- **Performance tests**: Before/after timing comparisons
- **Regression tests**: Existing Module Federation test suite
- **Stress tests**: Large applications with many shared modules

### 4. Rollback Plans
- Each phase can be independently rolled back
- Feature flags allow instant disable
- Original logic preserved as fallback
- Clear commit boundaries for easy reversion

## Testing Requirements per Phase

### Phase 1 Tests
```rust
#[test]
fn test_buildmeta_fields_serialization() {
    // Test BuildMeta with new fields serializes correctly
}

#[test] 
fn test_esm_shared_detection_buildmeta() {
    // Test that ESM shared modules get correct BuildMeta
}

#[test]
fn test_esm_imports_with_buildmeta_optimization() {
    // Test ESM imports work with BuildMeta lookup
}

#[test]
fn test_esm_performance_improvement() {
    // Benchmark ESM import performance before/after
}
```

### Phase 2 Tests
```rust
#[test]
fn test_commonjs_exports_buildmeta() {
    // Test CommonJS exports get correct BuildMeta
}

#[test]
fn test_consume_shared_exports_optimization() {
    // Test should_apply_to_module uses BuildMeta correctly
}

#[test]
fn test_commonjs_tree_shaking_preserved() {
    // Test tree-shaking annotations still work
}
```

## Implementation Timeline

- **Phase 1**: 2-3 days (ESM optimization + tests)
- **Phase 2**: 2 days (CommonJS exports + tests)  
- **Phase 3**: 1-2 days (CommonJS requires + tests)
- **Phase 4**: 1 day (cleanup + final tests)
- **Phase 5**: 1 day (performance validation)

**Total**: 7-9 days with thorough testing and validation

## Success Criteria per Phase

### Phase 1 Success
- ✅ All existing ESM tests pass
- ✅ BuildMeta fields serialize correctly
- ✅ ESM import performance improves measurably
- ✅ No regression in CommonJS functionality

### Phase 2 Success  
- ✅ All CommonJS export tests pass
- ✅ Tree-shaking annotations still generated
- ✅ `should_apply_to_module()` performance improves
- ✅ No regression in other functionality

### Phase 3 Success
- ✅ All CommonJS require tests pass
- ✅ Shared context detection still works
- ✅ `detect_consume_shared_context()` performance improves
- ✅ No regression in dependency resolution

### Final Success
- ✅ 44% Module Federation overhead eliminated
- ✅ 21.61% threejs regression eliminated  
- ✅ All existing Module Federation tests pass
- ✅ Linear performance scaling with shared modules
- ✅ Code complexity reduced by 90%

This incremental approach ensures we can safely implement the optimizations from `IMPLEMENTATION_PLAN.md` while maintaining system stability and having clear rollback points at each phase.