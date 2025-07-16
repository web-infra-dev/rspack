# ShareUsagePlugin Test

This test validates the ShareUsagePlugin functionality when it becomes available through the JavaScript API.

## Current Status

The ShareUsagePlugin is implemented in Rust (`rspack_plugin_mf/src/sharing/share_usage_plugin.rs`) but not yet exposed through the JavaScript API. This test directory contains the structure and validation logic ready for when the plugin becomes available.

## Test Structure

- `rspack.config.js` - Configuration with Module Federation and shared modules
- `index.js` - Main test file with imports to trigger ConsumeShared dependencies  
- `utils.js` - Utility module with additional shared imports
- `components.js` - Component module using React
- `validate-share-usage.js` - Validation logic for the generated `share-usage.json`
- `test.config.js` - Test configuration with post-build validation

## Expected Behavior

When ShareUsagePlugin is enabled, it should:

1. Generate a `share-usage.json` file in the output directory
2. Analyze ConsumeShared module usage and track:
   - `used_exports`: Actually used exports from shared modules
   - `unused_exports`: Imported but unused exports  
   - `possibly_unused_exports`: Exports with unclear usage patterns
   - `entry_module_id`: Module ID for the fallback module

## JSON Structure

After the metadata removal changes, the expected JSON structure is:

```json
{
  "lodash-es": {
    "used_exports": ["map", "filter", "isEmpty", "isArray", "isObject"],
    "unused_exports": ["uniq", "debounce"],
    "possibly_unused_exports": [],
    "entry_module_id": "42"
  },
  "react": {
    "used_exports": ["createElement"],
    "unused_exports": [],
    "possibly_unused_exports": [],
    "entry_module_id": "24"
  }
}
```

**Important**: `entry_module_id` should always contain a string module ID (like "42", "24") when a fallback module is found. It will only be `null` when no fallback module is found, which would be an error condition in normal Module Federation usage.

## Running the Test

To enable this test once ShareUsagePlugin is available:

1. Add ShareUsagePlugin to the JavaScript API bindings
2. Update `rspack.config.js` to include the ShareUsagePlugin
3. Run: `pnpm test --testNamePattern="share-usage-plugin"`

## Unit Tests

Unit tests for the ShareUsagePlugin data structures and serialization are available in the Rust code:

```bash
cargo test -p rspack_plugin_mf share_usage_plugin::tests
```