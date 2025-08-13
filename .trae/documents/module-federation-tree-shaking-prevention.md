# How Module Federation Prevents Tree-Shaking in Rspack

## Overview

Module Federation in Rspack prevents tree-shaking optimization as a conservative safety mechanism for shared modules. This behavior occurs through an indirect chain of interactions between the module federation plugin system and Rspack's dependency usage tracking, ultimately resulting in shared modules being marked with "unknown usage" state, which disables tree-shaking.

## Technical Mechanism

### The Core Problem

Tree-shaking relies on precise knowledge of which exports are actually used. However, module federation introduces runtime dynamics where:

1. **Shared modules** can be consumed by multiple applications
2. **Export usage** may not be determinable at build time
3. **Runtime loading** can introduce dependencies that static analysis cannot detect

To ensure safety, Rspack conservatively disables tree-shaking for module federation scenarios by marking exports with "unknown usage" state.

### Key Components

#### 1. FlagDependencyUsagePlugin

Location: `crates/rspack_plugin_javascript/src/dependency/flag_dependency_usage_plugin.rs`

This plugin is responsible for analyzing dependency usage and setting export usage states. The critical logic occurs when:

```rust
// When used_exports is empty, mark as unknown usage
if used_exports.is_empty() {
    export_info.set_used_in_unknown_way(compilation);
}
```

#### 2. Module Federation Dependencies

Module federation introduces several dependency types that interact with the usage tracking system:

- **ConsumeSharedDependency**: Located in `crates/rspack_plugin_mf/src/consume_shared_dependency.rs`
- **ConsumeSharedFallbackDependency**: Located in `crates/rspack_plugin_mf/src/consume_shared_fallback_dependency.rs`
- **ProvideSharedDependency**: Located in `crates/rspack_plugin_mf/src/provide_shared_dependency.rs`

#### 3. Default Dependency Behavior

Location: `crates/rspack_core/src/dependency_trait.rs`

The `Dependency` trait provides a default implementation for `get_referenced_exports()`:

```rust
fn get_referenced_exports(&self, _module_graph: &ModuleGraph, _runtime: Option<&RuntimeSpec>) -> Vec<ExtendedReferencedExport> {
    create_exports_object_referenced()
}
```

This default implementation calls `create_exports_object_referenced()`, which is defined in `crates/rspack_core/src/dependency/referenced_export.rs`:

```rust
pub fn create_exports_object_referenced() -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(vec![])]
}
```

## Technical Flow

### Step-by-Step Process

1. **Dependency Creation**: Module federation creates dependencies like `ConsumeSharedFallbackDependency`

2. **Export Reference Query**: `FlagDependencyUsagePlugin` calls `get_referenced_exports()` on these dependencies

3. **Default Implementation**: Since module federation dependencies don't override `get_referenced_exports()`, they use the default implementation

4. **Empty Export List**: The default implementation returns `ExtendedReferencedExport::Array(vec![])`, indicating the entire exports object is referenced but no specific exports are identified

5. **Usage State Decision**: `FlagDependencyUsagePlugin` processes this empty list:
   ```rust
   let used_exports = /* extract from ExtendedReferencedExport */;
   if used_exports.is_empty() {
       export_info.set_used_in_unknown_way(compilation);
   }
   ```

6. **Tree-Shaking Prevention**: Setting `UsageState::Unknown` prevents tree-shaking optimization

### Module Federation Specific Handling

#### ConsumeSharedPlugin

Location: `crates/rspack_plugin_mf/src/consume_shared_plugin.rs`

The plugin explicitly delegates usage state handling:

```rust
// Usage state copying is handled by FlagDependencyUsagePlugin
```

This comment appears multiple times throughout the file, indicating that module federation plugins intentionally rely on the core usage tracking system rather than implementing custom logic.

#### ShareUsagePlugin

Location: `crates/rspack_plugin_mf/src/share_usage_plugin.rs`

This plugin processes shared module usage but works in conjunction with `FlagDependencyUsagePlugin`:

```rust
// Collect used_exports from dependencies
let used_exports = dependency.get_referenced_exports(module_graph, runtime);
// Process ExtendedReferencedExport to extract export names
```

## Code Examples and File References

### Module Federation Module Types

**ConsumeSharedModule** (`crates/rspack_plugin_mf/src/consume_shared_module.rs`):
```rust
fn module_type(&self) -> &ModuleType {
    &ModuleType::ConsumeShared
}
```

**ProvideSharedModule** (`crates/rspack_plugin_mf/src/provide_shared_module.rs`):
```rust
fn module_type(&self) -> &ModuleType {
    &ModuleType::ProvideShared
}
```

### Dependency Type Definitions

**ConsumeSharedFallbackDependency**:
```rust
fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ConsumeSharedFallback
}
```

### Usage State Handling

**Export Usage Tracking** (`crates/rspack_plugin_mf/src/consume_shared_plugin.rs`):
```rust
// Set unknown_exports_provided based on fallback module
if fallback_module.get_exports_info(module_graph).other_exports_info.get_provided() == Some(true) {
    consume_shared_module.set_unknown_exports_provided(true);
}
```

## Why This Behavior is Necessary

### Safety Considerations

1. **Runtime Dynamics**: Module federation allows runtime loading of modules, making static analysis insufficient

2. **Cross-Application Sharing**: Shared modules may be used by multiple applications with different usage patterns

3. **Fallback Mechanisms**: Module federation includes fallback logic that can introduce unexpected dependencies

4. **Version Compatibility**: Different versions of shared modules may have different export signatures

### Conservative Approach

Rspack takes a conservative approach by:

- **Marking exports as unknown usage** when precise usage cannot be determined
- **Preserving all exports** in shared modules to ensure runtime compatibility
- **Preventing tree-shaking** that could break module federation functionality

## Conclusion

Module federation prevents tree-shaking in Rspack through an indirect mechanism where module federation dependencies use the default `get_referenced_exports()` implementation, which returns an empty export list. This triggers `FlagDependencyUsagePlugin` to mark exports with unknown usage state, effectively disabling tree-shaking as a safety measure for the dynamic nature of module federation.

This behavior ensures that shared modules remain fully functional across different applications and runtime scenarios, prioritizing correctness over optimization in the context of module federation.

## References

- `crates/rspack_plugin_javascript/src/dependency/flag_dependency_usage_plugin.rs` - Core usage tracking logic
- `crates/rspack_core/src/dependency_trait.rs` - Default dependency behavior
- `crates/rspack_core/src/dependency/referenced_export.rs` - Export reference utilities
- `crates/rspack_plugin_mf/src/consume_shared_plugin.rs` - Module federation consume logic
- `crates/rspack_plugin_mf/src/share_usage_plugin.rs` - Shared module usage tracking
- `crates/rspack_plugin_mf/src/consume_shared_module.rs` - ConsumeShared module implementation
- `crates/rspack_plugin_mf/src/consume_shared_fallback_dependency.rs` - Fallback dependency implementation