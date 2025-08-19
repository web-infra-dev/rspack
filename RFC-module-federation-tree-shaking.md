# RFC: Module Federation Tree-Shaking Support with External Usage Preservation

## Summary

This RFC describes the implementation of tree-shaking support for Module Federation shared modules, with a mechanism to preserve exports needed by external applications through usage data coordination.

## Motivation

Module Federation allows multiple independent applications to share code at runtime. However, tree-shaking in this context presents unique challenges:

1. **Cross-application dependencies**: Each application can only analyze its own usage, but shared modules may have exports needed by other applications
2. **Bundle size optimization**: Without tree-shaking, shared modules include all exports even when only a subset is used across all consumers
3. **Coordination challenge**: Applications need a way to communicate which exports they need from shared modules to prevent breaking runtime dependencies

## Detailed Design

The solution involves a two-part system:
1. **Usage Reporting**: Each application reports what it uses from shared modules
2. **Usage Preservation**: Each application preserves exports that other applications need

### Core Architecture

#### Data Flow
```
App A Build:
1. Analyzes its own usage of shared modules
2. Loads external-usage.json (what App B & C need)
3. Merges both data sets (true always wins)
4. Writes merged data to temporary file for FlagDependencyUsagePlugin
5. FlagDependencyUsagePlugin marks exports for preservation
6. Tree-shaking respects these marks
7. Emits share-usage.json asset (what App A uses, for others to consume)

App B Build:
1. Uses App A's share-usage.json as its external-usage.json
2. Repeats the same process
```

### 1. ShareUsagePlugin

Tracks usage of shared modules and coordinates with tree-shaking:

```rust
pub struct ShareUsagePlugin {
  options: ShareUsagePluginOptions,
}

pub struct ShareUsagePluginOptions {
  pub filename: String, // Output filename (default: "share-usage.json")
  pub external_usage: Option<ExternalUsageConfig>, // External apps' requirements
}
```

**Execution Flow:**
1. **optimize_dependencies hook** (runs BEFORE FlagDependencyUsagePlugin):
   - Analyzes local usage via `analyze_consume_shared_usage()`
   - Loads external usage data from `external-usage.json`
   - Merges both (true always wins - preserve if ANY source needs it)
   - Writes temporary file to context directory for FlagDependencyUsagePlugin

2. **after_process_assets hook** (runs AFTER optimization):
   - Generates final `share-usage.json` as a build asset
   - This file reports what THIS app uses (for other apps to consume)

**Output Format (share-usage.json):**
```json
{
  "treeShake": {
    "react": {
      "useState": true,      // This app uses useState
      "useEffect": false,    // This app doesn't use useEffect
      "useContext": true     // This app uses useContext
    }
  }
}
```

### 2. Enhanced Module Metadata

Extended `BuildMeta` structure to track Module Federation metadata:

```rust
pub struct BuildMeta {
  // ... existing fields ...
  
  // Module federation fields
  pub consume_shared_key: Option<String>,
  pub shared_key: Option<String>,
  pub is_shared_descendant: Option<bool>,
  pub effective_shared_key: Option<String>,
}
```

This metadata enables:
- Tracking which modules are shared dependencies
- Preserving share keys through the module graph
- Identifying module relationships for tree-shaking decisions

### 3. Export Metadata Propagation

The `ConsumeSharedPlugin` propagates export information from fallback modules to ConsumeShared modules:

```rust
fn copy_exports_from_fallback_to_consume_shared(
  module_graph: &mut ModuleGraph,
  fallback_id: &ModuleIdentifier,
  consume_shared_id: &ModuleIdentifier,
) -> Result<()>
```

This ensures ConsumeShared modules have accurate export information for analysis.

### 4. FlagDependencyUsagePlugin Integration

The `FlagDependencyUsagePlugin` reads the temporary share-usage.json file written by ShareUsagePlugin:

```rust
// During optimize_dependencies phase
let usage_path = context_path.join("share-usage.json");
if let Ok(content) = std::fs::read_to_string(&usage_path) {
  // Read the merged usage data (local + external)
  // Mark exports as Used/Unused based on the merged data
  for module with shared_key {
    if usage_data[shared_key][export_name] == true {
      mark_as_used(export_name);  // Preserve this export
    } else {
      mark_as_unused(export_name); // Safe to tree-shake
    }
  }
}
```

**Important:** The share-usage.json file read here is a TEMPORARY file containing merged data (local + external usage), not the final asset that gets emitted.

### 5. Provide Shared Plugin Enhancements

Updated `ProvideSharedPlugin` to:
- Track shared module keys in module metadata
- Propagate share information through the dependency graph
- Maintain module relationships for tree-shaking analysis

## Implementation Details

### Key Components and Their Roles

1. **ShareUsagePlugin** (`rspack_plugin_mf/src/sharing/share_usage_plugin.rs`)
   - **Purpose**: Bridge between local usage analysis and external requirements
   - **Hooks into**:
     - `optimize_dependencies`: Merges local + external usage, writes temp file
     - `after_process_assets`: Emits final share-usage.json asset
   - **Key methods**:
     - `analyze_consume_shared_usage()`: Determines local usage
     - `load_external_usage()`: Reads external-usage.json
     - Merge logic: True always wins (preserve if ANY source needs it)

2. **FlagDependencyUsagePlugin** (`rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`)
   - **Purpose**: Marks exports for tree-shaking based on usage data
   - **Reads**: Temporary share-usage.json from context directory
   - **Process**: 
     - For each module with a shared_key
     - Looks up usage in the merged data
     - Marks exports as Used (preserve) or Unused (tree-shake)

3. **ConsumeSharedPlugin** (`rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`)
   - **Enhancement**: Propagates export metadata from fallback to ConsumeShared modules
   - **Purpose**: Ensures accurate export information for usage analysis

4. **Module Metadata** (`rspack_core/src/module.rs`)
   - **New fields in BuildMeta**:
     - `consume_shared_key`: Share key for ConsumeShared modules
     - `shared_key`: Share key for shared modules
     - `effective_shared_key`: Inherited share key through dependency chain

### Test Coverage

Added test case: `tests/webpack-test/configCases/sharing/consume-shared-tree-shaking/`
- Validates tree-shaking behavior with shared modules
- Ensures share-usage.json generation
- Tests export preservation based on usage data

## Benefits

1. **Optimized Bundle Sizes**: Shared modules can be tree-shaken while preserving exports needed by remote applications
2. **Runtime Safety**: Export usage tracking ensures runtime compatibility across module boundaries
3. **Build Coordination**: Multiple builds can share usage information to optimize collectively
4. **Developer Experience**: Automatic tracking and reporting of shared module usage
5. **Backwards Compatible**: Opt-in feature that doesn't break existing Module Federation setups
6. **Cross-System Integration**: External usage preservation allows coordination with non-Rspack systems
7. **Flexible Preservation Strategies**: Support for union, intersection, and override merge strategies
8. **Conditional Preservation**: Exports can be preserved based on remote names or environments

## Migration Strategy

1. **Phase 1**: Deploy ShareUsagePlugin in analysis mode
   - Generate usage reports without affecting tree-shaking
   - Allow teams to review and validate usage data

2. **Phase 2**: Enable tree-shaking with usage data
   - Apply usage information to influence tree-shaking
   - Monitor for runtime issues

3. **Phase 3**: Cross-application coordination
   - Share usage reports between applications
   - Optimize shared modules globally

## External Usage Preservation

### Understanding the Two-File System

The implementation uses two distinct files with different purposes:

1. **INPUT: external-usage.json** (in project root)
   - Contains exports that OTHER applications need from shared modules
   - Read during build to know what to preserve for external consumers
   - Typically copied from other apps' share-usage.json outputs

2. **OUTPUT: share-usage.json** (in dist/build artifacts)
   - Reports what THIS application uses from shared modules
   - Generated as a build asset for OTHER applications to use
   - Other apps can use this as their external-usage.json

### Complete Build Flow Example

```
App A Build:
├── INPUT: external-usage.json (what App B & C need from shared modules)
├── PROCESS: 
│   1. ShareUsagePlugin analyzes App A's usage
│   2. Loads external-usage.json
│   3. Merges: preserves exports needed by App A OR Apps B/C
│   4. Writes temp file for FlagDependencyUsagePlugin
│   5. Tree-shaking preserves all marked exports
└── OUTPUT: dist/share-usage.json (what App A needs, for B & C to use)

App B Build:
├── INPUT: external-usage.json (copied from App A's share-usage.json)
├── PROCESS: [same as above]
└── OUTPUT: dist/share-usage.json (what App B needs)
```

### JSON Schema for External Usage Data

```typescript
interface ExternalUsageConfig {
  // Path to external usage data file(s)
  sources?: string[] | string;
  // Inline external usage data
  inline?: ExternalUsageData;
  // Merge strategy when multiple sources conflict
  mergeStrategy?: 'union' | 'intersection' | 'override';
}

interface ExternalUsageData {
  // Version of the schema
  version: '1.0';
  // Usage data organized by share key
  modules: {
    [shareKey: string]: {
      // List of exports that must be preserved
      preservedExports: string[] | '*';
      // Optional: Source system identifier
      source?: string;
      // Optional: Priority for merge conflicts (higher wins)
      priority?: number;
      // Optional: Conditions for preservation
      conditions?: {
        // Preserve only for specific remotes
        remotes?: string[];
        // Preserve only for specific environments
        environments?: string[];
      };
    };
  };
  // Optional: Global settings
  settings?: {
    // Default behavior for unlisted modules
    defaultPreservation?: 'all' | 'none' | 'auto';
    // Timestamp of when this data was generated
    timestamp?: string;
  };
}
```

### External Usage Data Example

```json
{
  "version": "1.0",
  "modules": {
    "react": {
      "preservedExports": ["useState", "useEffect", "useContext", "memo"],
      "source": "remote-app-1",
      "priority": 10
    },
    "react-dom": {
      "preservedExports": "*",
      "source": "remote-app-1"
    },
    "lodash": {
      "preservedExports": ["debounce", "throttle", "get", "set"],
      "source": "analytics-system",
      "conditions": {
        "remotes": ["analytics", "reporting"]
      }
    },
    "@company/ui-kit": {
      "preservedExports": ["Button", "Modal", "Form", "Input"],
      "source": "design-system",
      "priority": 20
    }
  },
  "settings": {
    "defaultPreservation": "auto",
    "timestamp": "2025-01-19T10:00:00Z"
  }
}
```

### Aggregated Usage Format

For coordinating across multiple applications, an aggregated format combines usage from multiple sources:

```json
{
  "version": "1.0",
  "aggregated": true,
  "sources": [
    {
      "id": "app1",
      "url": "https://app1.example.com/share-usage.json",
      "timestamp": "2025-01-19T10:00:00Z"
    },
    {
      "id": "app2",
      "url": "https://app2.example.com/share-usage.json",
      "timestamp": "2025-01-19T09:30:00Z"
    }
  ],
  "modules": {
    "react": {
      "preservedExports": {
        "useState": ["app1", "app2"],
        "useEffect": ["app1", "app2"],
        "useContext": ["app1"],
        "useMemo": ["app2"],
        "useCallback": ["app2"]
      },
      "totalSources": 2
    }
  }
}
```

## Configuration Example

```javascript
// webpack.config.js
const { ModuleFederationPlugin } = require('@rspack/core').container;
const { ShareUsagePlugin } = require('@rspack/core').sharing;

module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'app',
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true }
      }
    }),
    new ShareUsagePlugin({
      filename: 'share-usage.json',
      // External usage configuration
      externalUsage: {
        // Load external usage from files
        sources: [
          './external-usage/remote-app.json',
          'https://cdn.example.com/usage/aggregated.json'
        ],
        // Or provide inline configuration
        inline: {
          version: '1.0',
          modules: {
            'react': {
              preservedExports: ['useState', 'useEffect'],
              source: 'remote-app'
            }
          }
        },
        // Merge strategy when conflicts occur
        mergeStrategy: 'union' // Preserve all exports from all sources
      }
    })
  ],
  optimization: {
    usedExports: true,
    sideEffects: false
  }
};
```

### CLI Integration

The external usage data can also be provided via CLI:

```bash
# Single external usage file
rspack build --external-usage ./external-usage.json

# Multiple sources
rspack build --external-usage app1.json --external-usage app2.json

# Remote source
rspack build --external-usage https://cdn.example.com/usage.json
```

### Programmatic API

```javascript
const { analyzeExternalUsage } = require('@rspack/core').sharing;

// Analyze and merge multiple usage sources
async function buildWithExternalUsage() {
  const externalUsage = await analyzeExternalUsage({
    sources: [
      './local-usage.json',
      'https://remote.example.com/usage.json'
    ],
    mergeStrategy: 'union'
  });
  
  // Use in webpack config
  return {
    plugins: [
      new ShareUsagePlugin({
        externalUsage: { inline: externalUsage }
      })
    ]
  };
}
```

## Unresolved Questions

1. **Cross-repository coordination**: How should usage data be shared between independently deployed applications?
2. **Version compatibility**: How to handle usage data when shared module versions differ?
3. **Dynamic imports**: How to track usage for dynamically imported shared modules?

## Alternatives Considered

1. **Runtime detection**: Detect usage at runtime instead of build time
   - Pros: More accurate for dynamic usage
   - Cons: Performance overhead, complexity

2. **Manual configuration**: Require developers to manually specify preserved exports
   - Pros: Simple implementation
   - Cons: Error-prone, maintenance burden

3. **No tree-shaking for shared modules**: Disable tree-shaking entirely for shared modules
   - Pros: Guaranteed compatibility
   - Cons: Larger bundle sizes

## How It All Works Together

### The Complete Picture

1. **Each app reports its usage**: Via share-usage.json in build artifacts
2. **Each app preserves external needs**: Via external-usage.json input
3. **Coordination happens through file sharing**: Apps exchange their usage reports
4. **Tree-shaking is safe**: Only truly unused exports are removed

### Key Insights

- **share-usage.json is OUTPUT only**: It reports what this app uses for others
- **external-usage.json is INPUT only**: It tells us what others need preserved
- **The temporary file is internal**: ShareUsagePlugin → FlagDependencyUsagePlugin communication
- **True always wins**: If ANY source needs an export, it's preserved

### Result

This design enables optimal tree-shaking across Module Federation boundaries:
- Each application only includes exports it needs OR that others need
- No manual configuration of preserved exports required
- Automatic coordination through usage data files
- Safe runtime module sharing with minimal bundle size

## References

- [Webpack Module Federation Documentation](https://webpack.js.org/concepts/module-federation/)
- [Rspack Module Federation Implementation](https://github.com/web-infra-dev/rspack/tree/main/crates/rspack_plugin_mf)
- [Tree-shaking in Webpack](https://webpack.js.org/guides/tree-shaking/)