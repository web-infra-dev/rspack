# Rspack Export System Architecture

## Overview
This diagram shows the high-level architecture of Rspack's compilation pipeline, focusing on how export processing integrates with the overall build system.

## System Architecture

```
┌─────────────────────────────────────────────────────┐
│                 COMPILATION                         │
├─────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌────────────────┐  ┌─────────┐ │
│  │ Module Graph  │──│ Dependency     │──│ Parser  │ │
│  │               │  │ Tree           │  │         │ │
│  └───────────────┘  └────────────────┘  └─────────┘ │
│         │                    │                      │
│         ▼                    ▼                      │
│  ┌───────────────┐  ┌────────────────┐              │
│  │ ExportsInfo   │  │ Export         │              │
│  │ System        │  │ Dependencies   │              │
│  └───────────────┘  └────────────────┘              │
│         │                    │                      │
│         ▼                    ▼                      │
│  ┌───────────────┐  ┌────────────────┐              │
│  │ Usage         │  │ Template       │              │
│  │ Analysis      │  │ System         │              │
│  └───────────────┘  └────────────────┘              │
│         │                    │                      │
│         ▼                    ▼                      │
│  ┌───────────────┐  ┌────────────────┐              │
│  │ Tree Shaking  │  │ InitFragment   │              │
│  │ Optimization  │  │ System         │              │
│  └───────────────┘  └────────────────┘              │
│                             │                       │
│                             ▼                       │
│                    ┌────────────────┐               │
│                    │ Code           │               │
│                    │ Generation     │               │
│                    └────────────────┘               │
└─────────────────────────────────────────────────────┘
```

## Enhanced Flow with Debug Information

Based on the build debug output, the actual execution flow includes these detailed phases:

### Phase 1: Build Initialization (Rust Compilation: 6.83s)
- CLI setup and workspace resolution
- Rust crate compilation (`rspack_node` binding)
- JavaScript package building (0.02s - 0.31s)

### Phase 2: Module Discovery & Dependency Resolution
- **ESM Import Dependency Processing** (988+ entries observed)
- Module identifier generation: `javascript/esm|[absolute_path]`
- Parent-child module relationship mapping
- Lodash-es module dependency processing (heavy workload)

### Phase 3: Export System Processing
- ESM Export Imported Specifier Dependency handling
- Export mode resolution and validation
- Re-export chain tracking

### Phase 4: Plugin Execution Framework
- ESM dependency plugin processing
- Template system activation
- InitFragment creation and composition

### Phase 5: Code Generation (interrupted in debug log)
- Bundle creation
- Output format handling (CJS, ESM)
- File size optimization

## Key Components Details

### Module Graph
- Central repository for all module metadata
- Tracks cross-module dependencies and export relationships
- Maintains ExportsInfo and ExportInfo mappings

### Dependency Tree
- Hierarchical representation of module dependencies
- Includes ESM imports, exports, and re-exports
- Processes 144+ ESM import dependencies in typical builds

### ExportsInfo System
- Metadata tracking for exports across modules
- Usage state determination (Used/Unused/OnlyPropertiesUsed)
- Tree-shaking decision support

### Export Dependencies
- ESMExportSpecifierDependency
- ESMExportImportedSpecifierDependency  
- ESMExportExpressionDependency
- Re-export mode resolution

### Template System
- Code generation from dependency metadata
- JavaScript runtime function injection
- Bundle optimization patterns

### InitFragment System
- Initialization code composition
- Runtime globals management
- Final JavaScript bundle assembly

## Performance Characteristics

- **Rust compilation**: Major bottleneck (6.83s)
- **Module processing**: Scales with dependency count (988+ entries)
- **JavaScript builds**: Very fast (0.02s - 0.31s)
- **Memory usage**: Grows with module graph complexity