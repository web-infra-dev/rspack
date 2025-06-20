# ConsumeShared Usage Flagging Test

## What Was Implemented

The FlagDependencyUsagePlugin has been enhanced to properly handle ConsumeShared modules by treating them the same as normal modules for usage tracking.

### Key Changes Made

1. **Enhanced FlagDependencyUsagePlugin** (`crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`):
   - Added special detection for ConsumeShared modules
   - Added `process_consume_shared_module()` method that mirrors normal module processing
   - ConsumeShared modules now get proper usage state assignment

2. **Integration Points**:
   - Export provision from fallback module (already implemented in ConsumeSharedPlugin)
   - Usage tracking now works for ConsumeShared modules (newly implemented)
   - Tree-shaking can now eliminate unused ConsumeShared exports

### How It Works

```rust
// In FlagDependencyUsagePlugin::process_referenced_module()
if module.module_type() == &rspack_core::ModuleType::ConsumeShared {
  self.process_consume_shared_module(module_id, used_exports, runtime, force_side_effects, queue);
  return;
}
```

The `process_consume_shared_module()` method:
1. Processes specific export usage (marks exports as `Used` or `OnlyPropertiesUsed`)
2. Handles namespace usage (marks all exports as used in unknown way)
3. Applies mangling and inlining constraints
4. Supports nested export access
5. Handles side-effect-only usage

### Expected Behavior

With this implementation:

1. **ConsumeShared Modules**: Now participate fully in usage tracking
2. **Tree-Shaking**: Unused exports in ConsumeShared modules are eliminated
3. **Compatibility**: Maintains existing module federation proxy behavior
4. **Performance**: Efficient usage tracking with proper queue management

### Testing

To test this implementation:

1. Create a module federation setup with ConsumeShared modules
2. Import specific exports from ConsumeShared modules  
3. Build with tree-shaking enabled
4. Verify that unused exports are eliminated from ConsumeShared modules
5. Verify that used exports remain and function correctly

The enhancement ensures ConsumeShared modules work seamlessly with rspack's tree-shaking system while maintaining the proxy pattern required for module federation.