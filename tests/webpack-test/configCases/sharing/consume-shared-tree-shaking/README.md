# Module Federation Tree-Shaking with External Usage Test

This test validates that Module Federation can properly tree-shake shared modules while preserving exports needed by external systems.

## Test Structure

- **module.js**: Exports multiple values to test different scenarios:
  - **Directly used**: `used`, `alsoUsed`, `usedBoth` - imported and used in tests
  - **Unused**: `unused`, `alsoUnused`, `neverUsed` - should be tree-shaken
  - **Externally preserved**: `externallyUsed1`, `externallyUsed2`, `sharedUtility` - preserved via external-usage.json

- **external-usage.json**: Specifies exports to preserve for external systems using share keys

- **bootstrap.js**: The main test file that:
  - Uses `used`, `alsoUsed`, and `usedBoth` directly
  - Verifies that unused exports are tree-shaken (undefined)
  - Verifies that externally marked exports are preserved despite not being directly used

- **index.js**: Entry point that dynamically imports bootstrap.js (avoids eager loading)

## How It Works

1. The ModuleFederationPlugin automatically applies ShareUsagePlugin
2. ShareUsagePlugin analyzes what THIS app uses from shared modules
3. It loads `external-usage.json` to see what OTHER apps need preserved
4. Merges both into `share-usage.json`:
   - Local usage: exports this app actually imports (marked true)
   - External requirements: exports other apps need (also marked true)
   - Unused by anyone: safe to tree-shake (marked false)
5. FlagDependencyUsagePlugin reads `share-usage.json` during optimization
6. Tree-shaking preserves all exports marked as true, removes those marked as false

## Expected Behavior

**Available exports (✅):**
- `used`, `alsoUsed`, `usedBoth` - directly imported
- `externallyUsed1`, `externallyUsed2`, `sharedUtility` - preserved via external-usage.json

**Tree-shaken exports (❌):**
- `unused`, `alsoUnused`, `neverUsed` - not used anywhere and not marked for external preservation

This ensures that Module Federation builds can safely tree-shake while maintaining compatibility with external consumers.