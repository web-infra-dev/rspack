# Re-export Flow Processing Diagrams

This document contains focused architectural diagrams for Rspack's re-export processing system, extracted from the comprehensive analysis.

## 1. Re-export Processing Pipeline

```
┌──────────────┐    ┌─────────────────┐    ┌──────────────────┐
│ Parse        │ ─→ │ Dependency      │ ─→ │ Mode Resolution  │
│ export {...} │    │ Creation        │    │ Analysis         │
│ from './mod' │    │                 │    │                 │
└──────────────┘    └─────────────────┘    └──────────────────┘
                                                     │
┌──────────────┐    ┌─────────────────┐              │
│ Code         │ ←─ │ Template        │ ←────────────┘
│ Generation   │    │ Rendering       │
└──────────────┘    └─────────────────┘
```

## 2. Re-export Chain Resolution Mechanics

### Chain Tracking Flow
```
Module A: export { utils } from './B'
    ↓ (ESMExportImportedSpecifierDependency)
Module B: export { utils } from './C'
    ↓ (ESMExportImportedSpecifierDependency)
Module C: export const utils = {...}
    ↓ (ESMExportSpecifierDependency)
Final: A.utils → B.utils → C.utils (terminal)
```

### Chain Resolution Algorithm
```rust
fn resolve_export_chain(&self, name: &str, visited: &mut HashSet<ModuleId>) -> ExportTarget {
  if visited.contains(&self.module_id) {
    return ExportTarget::Circular;  // Circular detection
  }
  visited.insert(self.module_id);

  match self.get_target_for_export(name) {
    Some(ExportTarget::Module(target_module, target_name)) => {
      target_module.resolve_export_chain(target_name, visited)  // Recurse
    }
    Some(ExportTarget::Local(local_name)) => {
      ExportTarget::Terminal(self.module_id, local_name)        // Terminal
    }
    None => ExportTarget::Missing
  }
}
```

## 3. Export Mode Determination Logic

### Mode Resolution States
```
┌─────────────────┐
│ Export Analysis │
└─────────┬───────┘
          │
  ┌───────▼───────┐
  │ Is export     │
  │ used?         │
  └───┬───────────┘
      │
  ┌───┼────────────────┐
  │   │ YES            │ NO
  ▼   ▼                ▼
┌──────────┐    ┌──────────────┐
│Can be    │    │Remove export │
│inlined?  │    │(production)  │
└─┬────────┘    │Add comment   │
  │             │(development) │
┌─┼──┐          └──────────────┘
│YES│NO
▼   ▼
┌──────────┐ ┌─────────────┐
│Inline at │ │Generate     │
│usage     │ │property     │
│sites     │ │getter       │
└──────────┘ └─────────────┘
```

### Usage State Hierarchy
```
Unused (0) ──┐
             ├─→ OnlyPropertiesUsed (1) ──┐
             ├─→ NoInfo (2) ──────────────┼─→ Unknown (3) ──┐
             └─→ Used (4) ←───────────────┴─────────────────┘

State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

## 4. Re-export Dependency Handling Stages

### Processing Stages
```rust
pub enum InitFragmentStage {
  StageConstants,         // 0: const declarations
  StageAsyncBoundary,     // 1: async module boundary
  StageESMExports,        // 2: ESM export definitions ★
  StageESMImports,        // 3: ESM import statements
  StageProvides,          // 4: provided dependencies
  StageAsyncDependencies, // 5: async dependency handling
  StageAsyncESMImports,   // 6: async ESM imports
}
```

### Code Generation Flow
```
Dependencies              Templates              InitFragments           Final Code
     │                        │                      │                    │
┌────▼────┐              ┌────▼────┐            ┌────▼────┐          ┌────▼────┐
│ESMExport│ ─── render ─→│Template │ ── add ──→ │Fragment │ ── sort/│Generated│
│Specifier│              │Render   │            │Creation │  merge │JavaScript│
│Dependency│             │Logic    │            │         │   ──→  │Code     │
└─────────┘              └─────────┘            └─────────┘        └─────────┘
     │                        │                      │                    │
     │                  ┌─────▼─────┐           ┌────▼────┐               │
     │                  │Runtime    │           │Fragment │               │
     └──── metadata ──→ │Requirements│ ── add ─→│Contents │ ── concat ────┘
                        │Collection │           │Rendering│
                        └───────────┘           └─────────┘
```

## 5. Performance Characteristics

### Star Re-export Resolution Optimization
```rust
// Optimized star re-export resolution with early exits
fn resolve_star_reexports(&self, module_graph: &ModuleGraph) -> Vec<ExportItem> {
  let imported_module = module_graph.get_module_by_dependency_id(&self.id)?;
  let exports_info = module_graph.get_exports_info(&imported_module.identifier());

  let mut items = Vec::new();

  // Get all provided exports from imported module
  for (name, export_info) in exports_info.exports {
    // Skip 'default' and locally defined exports (performance)
    if name == "default" || is_locally_defined(&name) {
      continue;
    }

    // Check if export is provided and used (early exit)
    if export_info.provided == Some(ExportProvided::Provided) {
      items.push(ExportItem {
        name: name.clone(),
        ids: vec![name],
        hidden: false,
        checked: export_info.provided.is_some(),
        export_info: Some(export_info),
      });
    }
  }

  items
}
```

### Circular Re-export Detection
```
Module A: export * from './B'
Module B: export * from './A'  // Circular!

Detection Strategy:
1. Connection State Tracking: Each dependency connection tracks `active` state
2. Side Effect Analysis: Modules with side effects break cycles
3. Runtime Condition Filtering: Prevents infinite loops during generation
```

### Usage-Driven Optimization
```
┌────────────────┐    ┌─────────────────┐    ┌──────────────────┐
│ Entry Points   │ ─→ │ Dependency      │ ─→ │ Usage            │
│ (User imports) │    │ Analysis        │    │ Propagation      │
└────────────────┘    └─────────────────┘    └──────────────────┘
                              │                        │
                              ▼                        ▼
                      ┌─────────────────┐    ┌──────────────────┐
                      │ Export Info     │    │ Tree Shaking     │
                      │ Metadata        │    │ Decisions        │
                      └─────────────────┘    └──────────────────┘
```

## Key Architectural Insights

1. **Multi-Stage Processing**: Re-exports flow through distinct stages (parse → dependency → resolution → generation)

2. **Chain Resolution with Caching**: Recursive chain traversal with circular detection and visited set optimization

3. **Usage-Driven Generation**: Code generation decisions based on actual usage analysis, enabling aggressive tree-shaking

4. **Performance Optimizations**:
   - Early exit conditions in star re-export resolution
   - Circular dependency detection prevents infinite loops
   - Usage state caching reduces redundant analysis
   - Deterministic fragment ordering for consistent output

5. **Mode-Aware Processing**: Different handling for development vs production builds with debug information preservation