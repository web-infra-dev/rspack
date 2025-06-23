# RSPACK Usage Analysis and Tree-Shaking Decision Diagrams

This document extracts and focuses on the usage analysis workflows, decision trees, and optimization processes from the RSPACK export system analysis.

## Usage State Hierarchy and Transitions

### Usage State Enumeration
```rust
pub enum UsageState {
  Unused = 0,              // Dead code, can eliminate
  OnlyPropertiesUsed = 1,  // Object properties accessed: obj.prop
  NoInfo = 2,              // Analysis incomplete
  Unknown = 3,             // Runtime-determined usage
  Used = 4,                // Definitely used
}
```

### State Transition Flow
```
Unused ──┐
         ├─→ OnlyPropertiesUsed ──┐
         ├─→ NoInfo ──────────────┼─→ Unknown ──┐
         └─→ Used ←───────────────┴─────────────┘
```

**State Progression Logic:**
- `Unused` → Can transition to any higher state when usage is detected
- `OnlyPropertiesUsed` → Specific optimization for object property access
- `NoInfo` → Default state when analysis is incomplete
- `Unknown` → Fallback for runtime-determined usage
- `Used` → Terminal state indicating definite usage

## Usage Collection and Propagation Workflow

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

**Flow Stages:**
1. **Entry Points**: User imports and explicit dependencies
2. **Dependency Analysis**: Static analysis of import/export relationships
3. **Usage Propagation**: Tracking usage through module graph
4. **Export Info Metadata**: Centralized usage state management
5. **Tree Shaking Decisions**: Final optimization choices

## Tree-Shaking Decision Tree

```
                    Export Analysis
                          │
                          ▼
                  ┌─────────────────┐
                  │ Is export used? │
                  └─────────┬───────┘
                           │
              ┌─────────────┼─────────────┐
              │             │ YES         │ NO
              ▼             ▼             ▼
    ┌─────────────┐ ┌─────────────┐ ┌──────────────┐
    │Can be       │ │Generate     │ │Remove export │
    │inlined?     │ │export code  │ │(production)  │
    └─────┬───────┘ └─────────────┘ │Add comment   │
          │                         │(development) │
      ┌───┼───┐                     └──────────────┘
  YES │   │   │ NO
      ▼   │   ▼
┌──────────┐ ┌─────────────┐
│Inline    │ │Generate     │
│at usage  │ │property     │
│sites     │ │getter       │
└──────────┘ └─────────────┘
```

**Decision Logic:**
- **Used Export**: Generate export code or apply inlining optimization
- **Unused Export**: Remove in production, add comment in development
- **Inlining Check**: Determine if export can be inlined at usage sites

## Usage State Impact on Code Generation

```
Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

**Optimization Outcomes:**
- **Used**: Full export code generation
- **OnlyPropertiesUsed**: Property-specific optimizations
- **Unknown/NoInfo**: Conservative approach - generate code
- **Unused**: Elimination or development comments

## Side Effect Analysis Integration

### Connection State Determination
```rust
fn get_module_evaluation_side_effects_state(
  &self,
  module_graph: &ModuleGraph,
  connection_state_cache: &mut IdentifierMap<ConnectionState>,
) -> ConnectionState {
  // Check for obvious side effects
  if imported_module.build_meta().has_top_level_await
    || imported_module.build_info().has_module_side_effects {
    return ConnectionState::Active(true);
  }

  // Analyze export-specific side effects
  let exports_info = module_graph.get_exports_info(&imported_module.identifier());
  if exports_info.other_exports_info().get_used(runtime) > UsageState::Unused {
    return ConnectionState::Active(true);
  }

  // No side effects detected
  ConnectionState::Active(false)
}
```

### Side Effect Decision Flow
```
Module Analysis
      │
      ▼
┌─────────────────┐
│Has top-level    │ YES → ConnectionState::Active(true)
│await or side    │
│effects?         │
└─────┬───────────┘
      │ NO
      ▼
┌─────────────────┐
│Exports used >   │ YES → ConnectionState::Active(true)
│UsageState::     │
│Unused?          │
└─────┬───────────┘
      │ NO
      ▼
ConnectionState::Active(false)
```

## Export Info Data Structure and Tracking

```rust
pub struct ExportInfoData {
  name: Option<Atom>,                    // Export name
  used_name: Option<Atom>,              // Mangled name or None if unused
  target: HashMap<Option<DependencyId>, ExportInfoTargetValue>, // Re-export targets  
  provided: Option<ExportProvided>,      // Provision state
  can_mangle_provide: Option<bool>,      // Mangle safety when providing
  can_mangle_use: Option<bool>,         // Mangle safety when using
  terminal_binding: bool,               // Stops traversal
  exports_info: Option<ExportsInfo>,    // Nested exports
  used_in_runtime: Option<UstrMap<UsageState>>, // Per-runtime usage
}
```

### Export Provision States
```rust
pub enum ExportProvided {
  Provided,     // Statically confirmed: export const foo = 1
  NotProvided,  // Statically confirmed absent
  Unknown,      // Runtime-determined: CommonJS, dynamic imports
}
```

## Usage-Driven Code Generation Logic

```rust
// Core usage-driven generation pattern
match used_name {
  Some(UsedName::Normal(ref used_vec)) => {
    // Export is used - generate export code
    let return_value = format!("/* export {} */ {}", &dep.name, &dep.local);
    self.add_export_fragment(context, return_value, used_vec.last().unwrap());
  }
  Some(UsedName::Inlined(inlined_value)) => {
    // Export can be inlined - add comment only
    source.insert(
      dep.range().start,
      format!("/* inlined export {} */ ", &dep.name).as_str(),
      None,
    );
  }
  None => {
    // Export is unused - remove completely in production
    if !compilation.options.mode.is_development() {
      source.replace(dep.range().start, dep.range().end, "", None);
    } else {
      source.insert(
        dep.range().start,
        format!("/* unused export {} */ ", &dep.name).as_str(),
        None,
      );
    }
  }
}
```

### Code Generation Decision Matrix
```
UsedName State    │ Production Mode      │ Development Mode
─────────────────┼─────────────────────┼─────────────────────
Normal           │ Generate export      │ Generate export
Inlined          │ Inline at usage     │ Inline + comment
None (Unused)    │ Remove completely   │ Add unused comment
```

## Usage Propagation Mechanisms

### Propagation Triggers
1. **Direct Import**: `import { foo } from './module'`
2. **Re-export Chain**: `export { foo } from './module'`  
3. **Dynamic Access**: `module.foo` or `module['foo']`
4. **Property Iteration**: `Object.keys(module)`

### Propagation Algorithm
```
Entry Point Usage Detection
           │
           ▼
Module Graph Traversal
           │
           ▼ 
Dependency Relationship Analysis
           │
           ▼
Export Info State Update
           │
           ▼
Usage State Propagation to Dependencies
           │
           ▼
Tree-Shaking Decision Application
```

## Key Architectural Principles

1. **Conservative Safety**: Unknown states default to "used" to prevent breaking changes
2. **Runtime Awareness**: Different usage states per runtime environment
3. **Incremental Analysis**: Usage states can be updated as more information becomes available
4. **Side Effect Preservation**: Modules with side effects are never eliminated
5. **Development/Production Split**: Different behaviors based on compilation mode

## Performance Optimizations

### Usage-Based Optimizations
- **Dead Code Elimination**: Remove unused exports
- **Code Inlining**: Inline simple exports at usage sites
- **Property Access Optimization**: Optimize object property access patterns
- **Export Mangling**: Shorten export names based on usage frequency

### Analysis Caching
- **Connection State Cache**: Avoid redundant side effect analysis
- **Export Info Memoization**: Cache export metadata across builds
- **Usage State Persistence**: Maintain usage information between incremental builds