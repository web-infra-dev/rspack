# Research: Avoiding Cache Implementation for Shared Descendant Information

## Overview
This research examines alternatives to implementing a cache for `is_consume_shared_descendant` information by analyzing how Rspack stores computed metadata directly on modules and connections.

## 1. Direct Module Metadata Approaches

### BuildMeta Pattern Analysis
Looking at `/Users/bytedance/RustroverProjects/rspack/crates/rspack_core/src/module.rs`, Rspack extensively uses the `BuildMeta` struct to store computed flags and metadata:

```rust
#[cacheable]
#[derive(Debug, Default, Clone, Hash, Serialize)]
pub struct BuildMeta {
  pub strict_esm_module: bool,
  pub has_top_level_await: bool,
  pub esm: bool,
  pub exports_type: BuildMetaExportsType,
  pub default_object: BuildMetaDefaultObject,
  pub side_effect_free: Option<bool>,
  
  // Module Federation specific fields
  pub consume_shared_key: Option<String>,
  pub shared_key: Option<String>,
}
```

**Key Finding**: BuildMeta already has Module Federation-specific fields (`consume_shared_key`, `shared_key`), making it the perfect place for `is_shared_descendant`.

### Usage Patterns in Codebase

1. **Setting during build**: Many plugins set BuildMeta flags during the build phase
   ```rust
   // From rspack_plugin_wasm/src/parser_and_generator.rs
   parse_context.build_meta.has_top_level_await = true;
   parse_context.build_meta.exports_type = BuildMetaExportsType::Namespace;
   
   // From rspack_plugin_mf/src/sharing/provide_shared_plugin.rs
   module.build_meta_mut().shared_key = Some(config.share_key.clone());
   ```

2. **Direct property access**: Flags are accessed directly without caches
   ```rust
   // From flag_dependency_usage_plugin.rs
   matches!(module.build_meta().exports_type, BuildMetaExportsType::Unset)
   
   // From normal_module.rs  
   if Some(true) == self.build_meta().side_effect_free {
   ```

## 2. How Other Plugins Handle Similar Needs

### Flag Dependency Usage Plugin
The `FlagDependencyUsagePlugin` shows how computed information is handled:

- **Direct access**: Uses `module.build_meta().exports_type` directly
- **No caching**: Relies on direct property lookup
- **Context-specific logic**: Handles special cases like ConsumeShared modules with dedicated methods

### Flag Dependency Exports Plugin  
This plugin demonstrates computation during dependency analysis:

```rust
// From flag_dependency_exports_plugin.rs
fn process_exports_spec(&mut self, module_id: &ModuleIdentifier, ...) {
  // Computes export information and stores it directly
  exports_info.set_unknown_exports_provided(self.mg, ...);
}
```

**Pattern**: Computed information is stored directly in ExportsInfo structures, not cached separately.

## 3. Module Graph Connection Metadata

### Connection State Pattern
Connections store state information that's computed on-demand:

```rust
// From module.rs
fn get_side_effects_connection_state(
  &self,
  _module_graph: &ModuleGraph,
  _module_graph_cache: &ModuleGraphCacheArtifact, 
  _module_chain: &mut IdentifierSet,
  _connection_state_cache: &mut IdentifierMap<ConnectionState>,
) -> ConnectionState {
  ConnectionState::Active(true)
}
```

**Key Insight**: While there's a `connection_state_cache` parameter, the default implementation returns computed values directly.

## 4. Dependency-Time Computation Examples

### ConsumeShared Module Implementation
The `ConsumeSharedModule` shows how Module Federation computes information at creation time:

```rust
// From consume_shared_module.rs
impl Module for ConsumeSharedModule {
  fn get_consume_shared_key(&self) -> Option<String> {
    Some(self.options.share_key.clone())  // Direct property access
  }
}
```

### Module Federation Plugin Pattern
```rust
// From provide_shared_plugin.rs  
module.build_meta_mut().shared_key = Some(config.share_key.clone());

// From consume_shared_plugin.rs
module.build_meta_mut().consume_shared_key = Some(share_key);
```

**Pattern**: Module Federation plugins set metadata directly in BuildMeta during module creation/factorization.

## 5. Similar Cases: Module "Ancestry" Information

### Side Effects Analysis
```rust
// From normal_module.rs
fn get_side_effects_connection_state(...) -> ConnectionState {
  if let Some(side_effect_free) = self.factory_meta().and_then(|m| m.side_effect_free) {
    return ConnectionState::Active(!side_effect_free);
  }
  if Some(true) == self.build_meta().side_effect_free {
    return ConnectionState::Active(false);
  }
  // ... more complex logic
}
```

**Key Insight**: Complex module relationship information is computed on-demand, not pre-cached.

## 6. BuildMeta Usage Patterns Analysis

### Computation During Build Phase
```rust
// From rspack_plugin_json/src/lib.rs
build_meta.exports_type = BuildMetaExportsType::Default;
build_meta.default_object = BuildMetaDefaultObject::RedirectWarn { ignore: true };

// From rspack_core/src/normal_module_factory.rs  
side_effect_free: Some(true),  // Set during factory creation
```

### Direct Flag Access
```rust
// Multiple locations show direct access patterns:
module.build_meta().consume_shared_key.is_some()
module.build_meta().shared_key.is_some()  
module.build_meta().side_effect_free
```

## 7. Recommended Implementation Strategy

Based on the research, here's the optimal approach to avoid implementing a cache:

### Add BuildMeta Field
```rust
#[cacheable]
#[derive(Debug, Default, Clone, Hash, Serialize)]
pub struct BuildMeta {
  // ... existing fields
  
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_shared_descendant: Option<bool>,
}
```

### Compute During Dependency Analysis
Follow the Module Federation plugin pattern:

1. **During factorization**: Analyze module relationships
2. **Set flag directly**: Use `module.build_meta_mut().is_shared_descendant = Some(true)`
3. **Direct access**: Use `module.build_meta().is_shared_descendant == Some(true)`

### Integration Points
```rust
// In consume_shared_plugin.rs or similar
fn analyze_shared_descendants(&mut self, compilation: &mut Compilation) {
  for module_id in compilation.get_module_graph().modules().keys() {
    let is_descendant = self.compute_descendant_status(module_id, compilation);
    if let Some(module) = compilation.get_module_graph_mut().module_by_identifier_mut(&module_id) {
      module.build_meta_mut().is_shared_descendant = Some(is_descendant);
    }
  }
}
```

## 8. Benefits of This Approach

1. **Consistency**: Follows established Rspack patterns
2. **Performance**: Direct property access, no cache overhead
3. **Persistence**: BuildMeta is automatically serialized/cached
4. **Simplicity**: No additional data structures or cache invalidation
5. **Debuggability**: Information visible in module metadata

## 9. Precedent Examples

### Module Federation Already Uses This Pattern
- `consume_shared_key` in BuildMeta for ConsumeShared modules
- `shared_key` in BuildMeta for ProvideShared modules
- Direct property access throughout the codebase

### Other Computed Flags
- `side_effect_free`: Computed during build, stored directly
- `exports_type`: Analyzed and stored in BuildMeta
- `has_top_level_await`: Set during parsing, accessed directly

## 10. Conclusion

**Recommendation**: Implement `is_shared_descendant` as a `BuildMeta` field rather than a separate cache.

This approach:
- ✅ Follows established Rspack patterns
- ✅ Leverages existing Module Federation infrastructure  
- ✅ Provides direct, fast access
- ✅ Integrates with existing serialization/caching
- ✅ Requires minimal code changes
- ✅ Is consistent with how similar metadata is handled

The research shows that Rspack consistently favors direct property storage over separate caches for module metadata, making this the idiomatic and optimal solution.