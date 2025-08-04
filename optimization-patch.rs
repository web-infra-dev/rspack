// Performance optimization for is_consume_shared_descendant
// This patch adds caching to avoid repeated BFS traversals

use std::collections::HashMap;
use std::sync::Mutex;

// Add this to the compilation struct or a global cache
lazy_static::lazy_static! {
    static ref SHARED_MODULE_CACHE: Mutex<HashMap<ModuleIdentifier, bool>> = 
        Mutex::new(HashMap::new());
}

/// Optimized version with caching
fn is_consume_shared_descendant_cached(
    module_graph: &ModuleGraph, 
    module_id: &ModuleIdentifier
) -> bool {
    // Check cache first
    {
        let cache = SHARED_MODULE_CACHE.lock().unwrap();
        if let Some(&cached_result) = cache.get(module_id) {
            return cached_result;
        }
    }
    
    // Compute result using original algorithm
    let result = is_consume_shared_descendant_original(module_graph, module_id);
    
    // Cache the result
    {
        let mut cache = SHARED_MODULE_CACHE.lock().unwrap();
        cache.insert(*module_id, result);
    }
    
    result
}

/// Original algorithm (unchanged)
fn is_consume_shared_descendant_original(
    module_graph: &ModuleGraph, 
    module_id: &ModuleIdentifier
) -> bool {
    // Quick check: if the module itself has shared metadata or is a shared module type
    if let Some(module) = module_graph.module_by_identifier(module_id) {
        if module.build_meta().shared_key.is_some()
            || module.build_meta().consume_shared_key.is_some()
            || module.module_type() == &ModuleType::ConsumeShared
            || module.module_type() == &ModuleType::ProvideShared
        {
            return true;
        }
    }

    // Check if any issuer (module that imports this one) is a shared module
    // This uses a breadth-first search to find shared modules in the dependency chain
    let mut visited = HashSet::default();
    let mut queue = vec![*module_id];

    while let Some(current_id) = queue.pop() {
        if !visited.insert(current_id) {
            continue;
        }

        for connection in module_graph.get_incoming_connections(&current_id) {
            if let Some(issuer_id) = connection.original_module_identifier {
                if let Some(issuer_module) = module_graph.module_by_identifier(&issuer_id) {
                    // If we find a shared module in the chain, this module should get PURE annotations
                    if issuer_module.build_meta().shared_key.is_some()
                        || issuer_module.build_meta().consume_shared_key.is_some()
                        || issuer_module.module_type() == &ModuleType::ConsumeShared
                        || issuer_module.module_type() == &ModuleType::ProvideShared
                    {
                        return true;
                    }

                    // Continue searching up the chain
                    queue.push(issuer_id);
                }
            }
        }
    }

    false
}

// Alternative: Per-compilation cache (preferred approach)
// Add this field to Compilation struct:
struct Compilation {
    // ... existing fields
    shared_module_cache: RefCell<HashMap<ModuleIdentifier, bool>>,
}

impl Compilation {
    fn is_consume_shared_descendant_cached(&self, module_id: &ModuleIdentifier) -> bool {
        // Check cache
        let cache = self.shared_module_cache.borrow();
        if let Some(&result) = cache.get(module_id) {
            return result;
        }
        drop(cache);
        
        // Compute and cache
        let result = self.is_consume_shared_descendant_uncached(module_id);
        self.shared_module_cache.borrow_mut().insert(*module_id, result);
        result
    }
    
    // Clear cache when module graph changes
    fn invalidate_shared_module_cache(&self) {
        self.shared_module_cache.borrow_mut().clear();
    }
}

/*
PERFORMANCE IMPACT ESTIMATE:

Before optimization:
- O(V + E) per import statement 
- For 1000 modules with 5000 imports: 5000 * (1000 + edges) operations
- Roughly 5,000,000+ graph traversal operations

After optimization:
- O(1) cache lookup for repeated modules
- O(V + E) only for first access per module
- For same scenario: ~1000 unique computations + 4000 cache hits
- Roughly 1,000,000 operations (5x speedup)

MEMORY COST:
- HashMap<ModuleIdentifier, bool>: ~8 bytes per module
- For 1000 modules: ~8KB memory overhead
- Negligible compared to build time savings
*/