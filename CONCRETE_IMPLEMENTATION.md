# Concrete Implementation: Eliminate BFS Performance Bottleneck

## Summary
Replace expensive O(V+E) BFS traversal on every import with O(1) cache lookup by following Rspack's established ModuleGraphCacheArtifact pattern.

## Changes Required

### 1. Add Cache to ModuleGraphCacheArtifact

**File**: `crates/rspack_core/src/cache/persistent/occasion/make/module_graph.rs`

```rust
// Add this import
use rustc_hash::FxHashMap;

// Add this type alias
pub type SharedDescendantCache = Arc<RwLock<FxHashMap<ModuleIdentifier, bool>>>;

// Modify ModuleGraphCacheArtifactInner struct
pub struct ModuleGraphCacheArtifactInner {
    freezed: AtomicBool,
    get_mode_cache: GetModeCache,
    determine_export_assignments_cache: DetermineExportAssignmentsCache,
    get_exports_type_cache: GetExportsTypeCache,
    get_side_effects_connection_state_cache: GetSideEffectsConnectionStateCache,
    // ADD THIS LINE:
    shared_descendant_cache: SharedDescendantCache,
}

// Add to the new() implementation
impl ModuleGraphCacheArtifactInner {
    pub fn new() -> Self {
        Self {
            freezed: AtomicBool::new(false),
            get_mode_cache: GetModeCache::default(),
            determine_export_assignments_cache: DetermineExportAssignmentsCache::default(),
            get_exports_type_cache: GetExportsTypeCache::default(),
            get_side_effects_connection_state_cache: GetSideEffectsConnectionStateCache::default(),
            // ADD THIS LINE:
            shared_descendant_cache: Arc::new(RwLock::new(FxHashMap::default())),
        }
    }
    
    // ADD THIS METHOD:
    pub fn get_shared_descendant_cache(&self) -> &SharedDescendantCache {
        &self.shared_descendant_cache
    }
}
```

### 2. Add Pre-computation to ConsumeSharedPlugin

**File**: `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`

```rust
// Add these imports at the top
use std::collections::{HashSet, VecDeque};
use rustc_hash::FxHashMap;

// Add this implementation to ConsumeSharedPlugin
impl ConsumeSharedPlugin {
    /// Pre-compute shared descendant information during finish_modules
    /// Follows the same pattern as FlagDependencyExportsPlugin
    fn flag_shared_descendants(compilation: &mut Compilation) -> Result<()> {
        let module_graph = compilation.get_module_graph();
        let cache = compilation.module_graph_cache_artifact.get_shared_descendant_cache();
        
        // Clear existing cache
        cache.write().unwrap().clear();
        
        // Step 1: Find directly shared modules
        let mut shared_modules = Vec::new();
        for (module_id, module) in module_graph.modules() {
            if Self::is_directly_shared_module(module) {
                cache.write().unwrap().insert(*module_id, true);
                shared_modules.push(*module_id);
            }
        }
        
        // Step 2: BFS to mark all descendants (ONE TIME ONLY)
        let mut queue = VecDeque::from(shared_modules);
        let mut visited = HashSet::new();
        
        while let Some(current_id) = queue.pop_front() {
            if !visited.insert(current_id) {
                continue;
            }
            
            for connection in module_graph.get_outgoing_connections(&current_id) {
                let target_id = connection.module_identifier();
                
                // Mark as shared descendant if not already marked
                let mut cache_write = cache.write().unwrap();
                if !cache_write.contains_key(target_id) {
                    cache_write.insert(*target_id, true);
                    drop(cache_write);
                    queue.push_back(*target_id);
                }
            }
        }
        
        // Step 3: Mark all remaining modules as non-descendants
        let cache_read = cache.read().unwrap();
        let cached_modules: HashSet<ModuleIdentifier> = cache_read.keys().copied().collect();
        drop(cache_read);
        
        let mut cache_write = cache.write().unwrap();
        for (module_id, _) in module_graph.modules() {
            if !cached_modules.contains(module_id) {
                cache_write.insert(*module_id, false);
            }
        }
        
        Ok(())
    }
    
    fn is_directly_shared_module(module: &dyn Module) -> bool {
        module.build_meta().shared_key.is_some()
            || module.build_meta().consume_shared_key.is_some()
            || module.module_type() == &ModuleType::ConsumeShared
            || module.module_type() == &ModuleType::ProvideShared
    }
}

// Modify the existing finish_modules method
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
    // ... existing logic ...
    
    // ADD THIS CALL:
    Self::flag_shared_descendants(compilation)?;
    
    Ok(())
}
```

### 3. Replace Expensive Runtime Function

**File**: `crates/rspack_core/src/dependency/runtime_template.rs`

```rust
// REPLACE the entire is_consume_shared_descendant function (lines 17-59)
// with this O(1) implementation:

/// Check if a module is part of a shared bundle by checking the cached result
fn is_consume_shared_descendant(module_graph: &ModuleGraph, module_id: &ModuleIdentifier) -> bool {
    // This function now needs access to the cache, so we need to modify the signature
    // For now, keep the same signature and add a fallback
    false  // Temporary - see next step for proper integration
}

// BETTER: Add a new function that uses the cache
fn is_consume_shared_descendant_cached(
    compilation: &Compilation,
    module_id: &ModuleIdentifier,
) -> bool {
    let cache = compilation.module_graph_cache_artifact.get_shared_descendant_cache();
    cache.read().unwrap().get(module_id).copied().unwrap_or(false)
}
```

### 4. Update import_statement Function

**File**: `crates/rspack_core/src/dependency/runtime_template.rs`

```rust
// Modify the import_statement function (around line 450) to use cached version
pub fn import_statement(
    module: &dyn Module,
    compilation: &Compilation,  // Add compilation parameter
    runtime_requirements: &mut RuntimeGlobals,
    id: &DependencyId,
    request: &str,
    update: bool,
) -> (String, String) {
    // ... existing logic until pure annotation check ...
    
    // REPLACE this section (lines 450-457):
    // let is_pure = if let Some(module_identifier) = compilation
    //     .get_module_graph()
    //     .module_identifier_by_dependency_id(id)
    // {
    //     is_consume_shared_descendant(&compilation.get_module_graph(), module_identifier)
    // } else {
    //     false
    // };
    
    // WITH:
    let is_pure = if let Some(module_identifier) = compilation
        .get_module_graph()
        .module_identifier_by_dependency_id(id)
    {
        is_consume_shared_descendant_cached(compilation, module_identifier)
    } else {
        false
    };
    
    // ... rest of function unchanged ...
}
```

### 5. Update Function Signatures

You'll need to update all callers of `import_statement` to pass the `compilation` parameter. This includes:

**Files to update**:
- Various dependency plugins that call `import_statement`
- Look for calls like `import_statement(module, compilation, ...)` and add the compilation parameter

## Testing

### 1. Add Unit Test

**File**: `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shared_descendant_cache_consistency() {
        // Create test compilation with Module Federation setup
        let mut compilation = create_test_compilation_with_mf();
        
        // Run our pre-computation
        ConsumeSharedPlugin::flag_shared_descendants(&mut compilation).unwrap();
        
        // Verify results match original BFS for all modules
        let cache = compilation.module_graph_cache_artifact.get_shared_descendant_cache();
        let cache_read = cache.read().unwrap();
        
        for (module_id, _) in compilation.get_module_graph().modules() {
            let cached_result = cache_read.get(module_id).copied().unwrap_or(false);
            let bfs_result = is_consume_shared_descendant_original(&compilation.get_module_graph(), module_id);
            
            assert_eq!(cached_result, bfs_result, "Mismatch for module {:?}", module_id);
        }
    }
}
```

## Performance Impact

### Before
- **Per import**: O(V + E) BFS traversal
- **Threejs**: ~5000 imports × ~1000 modules = ~5M operations
- **Time**: Expensive graph traversal on every import

### After  
- **Pre-computation**: O(V + E) once during finish_modules
- **Per import**: O(1) hash map lookup
- **Threejs**: ~1000 operations total + 5000 O(1) lookups
- **Speedup**: ~5000x faster for import generation

### Memory Overhead
- **Additional memory**: ~16 bytes per module for cache
- **Threejs**: ~16KB additional memory
- **Trade-off**: Excellent (tiny memory cost for huge performance gain)

## Migration Strategy

1. **Phase 1**: Add cache infrastructure (low risk)
2. **Phase 2**: Add pre-computation to finish_modules
3. **Phase 3**: Replace runtime function with cache lookup
4. **Phase 4**: Update function signatures and callers
5. **Phase 5**: Test and validate performance improvement

## Rollback Plan

If issues arise:
1. Remove cache lookup from import_statement
2. Restore original is_consume_shared_descendant function
3. Remove pre-computation from finish_modules
4. Remove cache from ModuleGraphCacheArtifact

The changes are designed to be additive and easily reversible.