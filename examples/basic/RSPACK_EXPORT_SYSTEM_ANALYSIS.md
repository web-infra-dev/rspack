# Rspack Export System Deep Dive Analysis

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Module Graph and Dependency Tree](#module-graph-and-dependency-tree)
4. [Export Dependency Types](#export-dependency-types)
5. [Re-export Flow Analysis](#re-export-flow-analysis)
6. [ExportInfo Metadata System](#exportinfo-metadata-system)
7. [Code Generation Pipeline](#code-generation-pipeline)
8. [Tree Shaking Integration](#tree-shaking-integration)
9. [ConsumeShared and Module Federation](#consumeshared-and-module-federation)
10. [Performance Analysis and Optimizations](#performance-analysis-and-optimizations)
11. [Error Handling and Diagnostics](#error-handling-and-diagnostics)
12. [Runtime Behavior and Generated Code](#runtime-behavior-and-generated-code)
13. [Plugin System Integration](#plugin-system-integration)
14. [Advanced Use Cases and Edge Cases](#advanced-use-cases-and-edge-cases)
15. [Debugging and Troubleshooting](#debugging-and-troubleshooting)
16. [Best Practices and Patterns](#best-practices-and-patterns)
17. [Comparison with Other Bundlers](#comparison-with-other-bundlers)
18. [Visualizations and Diagrams](#visualizations-and-diagrams)

## Overview

Rspack's export system is a sophisticated multi-layered architecture that handles ES module exports, CommonJS exports, and re-exports with advanced optimization capabilities including tree shaking, name mangling, and Module Federation support.

### Key Components:

- **Module Graph**: Central dependency and export tracking
- **Dependency System**: Export dependency types and relationships
- **ExportInfo System**: Export metadata and usage tracking
- **Template System**: Code generation from dependencies
- **InitFragment System**: Initialization code composition
- **Runtime System**: JavaScript runtime function injection

---

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

---

## Module Graph and Dependency Tree

### Module Graph Structure

The `ModuleGraph` is the central data structure that tracks all modules and their relationships:

```rust
pub struct ModuleGraphPartial {
  // Core storage
  modules: IdentifierMap<Option<BoxModule>>,
  module_graph_modules: IdentifierMap<Option<ModuleGraphModule>>,
  connections: HashMap<DependencyId, Option<ModuleGraphConnection>>,

  // Export-specific data
  exports_info_map: UkeyMap<ExportsInfo, ExportsInfoData>,
  export_info_map: UkeyMap<ExportInfo, ExportInfoData>,
}
```

### Dependency Tree Visualization

```
Module A (entry.js)
├── ESMImportDependency → Module B
├── ESMExportSpecifierDependency
│   ├── export { foo }
│   └── ExportInfo: { name: "foo", used: true, can_mangle: true }
└── ESMExportImportedSpecifierDependency → Module C
    ├── export { bar } from './c'
    └── Creates re-export chain: A.bar → C.bar

Module B (lib.js)
├── ESMExportSpecifierDependency
│   ├── export { util }
│   └── ExportInfo: { name: "util", used: false, can_mangle: true }
└── ESMExportExpressionDependency
    ├── export default class Helper
    └── ExportInfo: { name: "default", used: true, terminal_binding: true }

Module C (shared.js)
├── ESMExportImportedSpecifierDependency → Module D
│   ├── export * from './d'  (Star re-export)
│   └── Creates dynamic re-export chain
└── ESMExportSpecifierDependency
    ├── export { bar }
    └── ExportInfo: { name: "bar", used: true, target: self }
```

### Module-to-Dependency-to-Export Relationship

```
┌─────────────┐    creates     ┌──────────────────────┐    provides    ┌─────────────┐
│   Module    │ ──────────────→│   Export             │ ─────────────→ │ ExportInfo  │
│   (JS File) │                │   Dependency         │                │ (Metadata)  │
└─────────────┘                └──────────────────────┘                └─────────────┘
      │                                 │                                       │
      │ contains                        │ generates                             │ tracks
      ▼                                 ▼                                       ▼
┌─────────────┐                ┌──────────────────────┐                ┌─────────────┐
│ Source Code │                │ Template +           │                │ Usage State │
│ export {...}│                │ InitFragment         │                │ Provided    │
└─────────────┘                └──────────────────────┘                │ Mangle Info │
                                        │                               └─────────────┘
                                        ▼
                               ┌──────────────────────┐
                               │ Generated            │
                               │ JavaScript Code      │
                               └──────────────────────┘
```

---

## Export Dependency Types

### ESM Export Dependencies

#### 1. ESMExportSpecifierDependency

**Purpose**: Handles `export { foo, bar as baz }` statements

```rust
pub struct ESMExportSpecifierDependency {
  span: DependencySpan,
  name: Atom,        // export name ("foo", "baz")
  local: Atom,       // local variable name ("foo", "bar")
  inline: bool,      // can be inlined for optimization
}
```

**Generated Code**:

```javascript
// Input: export { foo, bar as baz }
__webpack_require__.d(__webpack_exports__, {
	foo: function () {
		return foo;
	},
	baz: function () {
		return bar;
	}
});
```

#### 2. ESMExportImportedSpecifierDependency

**Purpose**: Handles re-exports like `export { x } from './mod'`

**Complex Export Modes**:

```rust
pub enum ExportMode {
  Missing,                           // Export not found
  Unused(ExportModeUnused),         // Tree-shaken out
  EmptyStar(ExportModeEmptyStar),   // export * with no exports
  ReexportDynamicDefault(..),       // Dynamic default re-export
  ReexportNamedDefault(..),         // Named default re-export
  ReexportNamespaceObject(..),      // export * as ns
  ReexportFakeNamespaceObject(..),  // Fake namespace for CommonJS
  ReexportUndefined(..),            // Re-export undefined value
  NormalReexport(..),               // Standard re-export
  DynamicReexport(..),              // Runtime re-export loop
}
```

**Mode Resolution Flow**:

```
Input: export { foo } from './bar'
  ↓
Analyze imported module's exports
  ↓
Is 'foo' statically analyzable?
  ├─ Yes → NormalReexport mode
  │   └─ Generate: function() { return bar_module.foo; }
  └─ No → DynamicReexport mode
      └─ Generate: Runtime property access loop
```

#### 3. ESMExportExpressionDependency

**Purpose**: Handles `export default expression`

```rust
pub struct ESMExportExpressionDependency {
  range: DependencyRange,
  prefix: String,        // "const __DEFAULT__ = " or "var __DEFAULT__ = "
  suffix: String,        // ";"
  range_decl: Option<DependencyRange>, // For function/class declarations
}
```

### CommonJS Export Dependencies

#### 1. CommonJsExportsDependency

```rust
pub enum CommonJsExportsDependency {
  Exports(ExportsBase),      // exports.foo =
  ModuleExports(ExportsBase), // module.exports.foo =
  This(ExportsBase),         // this.foo = (in modules)
}
```

### Export Dependency Hierarchy

```
                    Dependency (trait)
                         │
            ┌────────────┼────────────┐
            │                         │
    ModuleDependency           DependencyCodeGeneration
            │                         │
            │                         │
┌───────────┼───────────┐            │
│           │           │            │
ESM       CommonJS    External       │
Exports   Exports     Modules        │
│           │           │            │
├─ ESMExportSpecifierDependency ─────┤
├─ ESMExportImportedSpecifierDep ────┤
├─ ESMExportExpressionDependency ────┤
├─ CommonJsExportsDependency ────────┤
└─ ExportInfoDependency ─────────────┤
```

---

## Re-export Flow Analysis

### Re-export Processing Pipeline

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

### Star Re-export Resolution Algorithm

```rust
// Simplified star re-export resolution
fn resolve_star_reexports(&self, module_graph: &ModuleGraph) -> Vec<ExportItem> {
  let imported_module = module_graph.get_module_by_dependency_id(&self.id)?;
  let exports_info = module_graph.get_exports_info(&imported_module.identifier());

  let mut items = Vec::new();

  // Get all provided exports from imported module
  for (name, export_info) in exports_info.exports {
    // Skip 'default' and locally defined exports
    if name == "default" || is_locally_defined(&name) {
      continue;
    }

    // Check if export is provided and used
    if export_info.provided == Some(ExportProvided::Provided) {
      items.push(ExportItem {
        name: name.clone(),
        ids: vec![name],
        hidden: false, // May be set by conflict analysis
        checked: export_info.provided.is_some(),
        export_info: Some(export_info),
      });
    }
  }

  items
}
```

### Re-export Chain Tracking

```
Module A: export { utils } from './B'
    ↓ (ESMExportImportedSpecifierDependency)
Module B: export { utils } from './C'
    ↓ (ESMExportImportedSpecifierDependency)
Module C: export const utils = {...}
    ↓ (ESMExportSpecifierDependency)
Final: A.utils → B.utils → C.utils (terminal)
```

**Chain Resolution with Caching**:

```rust
fn resolve_export_chain(&self, name: &str, visited: &mut HashSet<ModuleId>) -> ExportTarget {
  if visited.contains(&self.module_id) {
    return ExportTarget::Circular;
  }
  visited.insert(self.module_id);

  match self.get_target_for_export(name) {
    Some(ExportTarget::Module(target_module, target_name)) => {
      target_module.resolve_export_chain(target_name, visited)
    }
    Some(ExportTarget::Local(local_name)) => {
      ExportTarget::Terminal(self.module_id, local_name)
    }
    None => ExportTarget::Missing
  }
}
```

### Circular Re-export Handling

```
Module A: export * from './B'
Module B: export * from './A'  // Circular!
```

**Detection and Resolution**:

1. **Connection State Tracking**: Each dependency connection tracks `active` state
2. **Side Effect Analysis**: Modules with side effects break cycles
3. **Runtime Condition Filtering**: Prevents infinite loops during generation

---

## ExportInfo Metadata System

### ExportInfo Data Structure

```rust
pub struct ExportInfoData {
  name: Option<Atom>,                    // "foo" or None for unknown
  used_name: Option<Atom>,              // Mangled name or None if unused
  target: HashMap<Option<DependencyId>, ExportInfoTargetValue>, // Re-export targets
  provided: Option<ExportProvided>,      // Provided | NotProvided | Unknown
  can_mangle_provide: Option<bool>,      // Safe to mangle when providing
  can_mangle_use: Option<bool>,         // Safe to mangle when using
  terminal_binding: bool,               // Final binding (stops traversal)
  exports_info: Option<ExportsInfo>,    // Nested exports for objects
  used_in_runtime: Option<UstrMap<UsageState>>, // Per-runtime usage
}
```

### Usage State Hierarchy

```rust
pub enum UsageState {
  Unused = 0,              // Dead code, can eliminate
  OnlyPropertiesUsed = 1,  // Object properties accessed: obj.prop
  NoInfo = 2,              // Analysis incomplete
  Unknown = 3,             // Runtime-determined usage
  Used = 4,                // Definitely used
}
```

**State Transitions**:

```
Unused ──┐
         ├─→ OnlyPropertiesUsed ──┐
         ├─→ NoInfo ──────────────┼─→ Unknown ──┐
         └─→ Used ←───────────────┴─────────────┘
```

### Export Provision Tracking

```rust
pub enum ExportProvided {
  Provided,     // Statically confirmed: export const foo = 1
  NotProvided,  // Statically confirmed absent
  Unknown,      // Runtime-determined: CommonJS, dynamic imports
}
```

### Usage Collection Flow

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

---

## Code Generation Pipeline

### Template System Architecture

```rust
pub trait DependencyTemplate: Debug + Sync + Send {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  );
}
```

### Template Context Flow

```rust
pub struct TemplateContext<'a, 'b, 'c> {
  pub compilation: &'a Compilation,           // Global compilation state
  pub module: &'a dyn Module,                // Current module being processed
  pub runtime_requirements: &'a mut RuntimeGlobals, // Required runtime functions
  pub init_fragments: &'a mut ModuleInitFragments<'b>, // Initialization fragments
  pub runtime: Option<&'a RuntimeSpec>,      // Target runtime environment
  pub concatenation_scope: Option<&'c mut ConcatenationScope>, // Module concat
  pub data: &'a mut CodeGenerationData,      // Additional generation data
}
```

### InitFragment System

#### Fragment Stages and Processing Order

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

#### ESMExportInitFragment - Core Export Generator

```rust
impl<C: InitFragmentRenderContext> InitFragment<C> for ESMExportInitFragment {
  fn contents(mut self: Box<Self>, context: &mut C) -> Result<InitFragmentContents> {
    // Declare runtime requirements
    context.add_runtime_requirements(RuntimeGlobals::EXPORTS);
    context.add_runtime_requirements(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);

    // Sort exports for deterministic output
    self.export_map.sort_by(|a, b| a.0.cmp(&b.0));

    // Process each export with ConsumeShared macro support
    let exports = format!(
      "{{\n  {}\n}}",
      self.export_map.iter()
        .map(|(name, value)| {
          let prop = property_name(name)?;
          let getter_func = context.returning_function(value, "");

          // Handle ConsumeShared tree-shaking macros
          if value.contains("@common:if") && value.contains("@common:endif") {
            self.process_consume_shared_export(prop, value, context)
          } else {
            Ok(format!("{prop}: {getter_func}"))
          }
        })
        .collect::<Result<Vec<_>>>()?
        .join(",\n  ")
    );

    Ok(InitFragmentContents {
      start: format!(
        "{}({}, {});",
        RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
        self.exports_argument,
        exports
      ),
      end: None,
    })
  }
}
```

### Code Generation Flow Diagram

```
Dependencies                Templates              InitFragments           Final Code
     │                          │                      │                    │
┌────▼────┐                ┌────▼────┐            ┌────▼────┐          ┌────▼────┐
│ESMExport│ ─── render ──→ │Template │ ── add ──→ │Fragment │ ── sort/│Generated│
│Specifier│                │Render   │            │Creation │  merge │JavaScript│
│Dependency│               │Logic    │            │         │   ──→  │Code     │
└─────────┘                └─────────┘            └─────────┘        └─────────┘
     │                          │                      │                    │
     │                    ┌─────▼─────┐           ┌────▼────┐               │
     │                    │Runtime    │           │Fragment │               │
     └──── metadata ────→ │Requirements│ ── add ─→│Contents │ ── concat ────┘
                          │Collection │           │Rendering│
                          └───────────┘           └─────────┘
```

### Runtime Requirements System

```rust
bitflags! {
  impl RuntimeGlobals: u128 {
    // Export-related runtime functions
    const EXPORTS = 1 << 44;                    // "__webpack_exports__"
    const DEFINE_PROPERTY_GETTERS = 1 << 38;    // "__webpack_require__.d"
    const HAS_OWN_PROPERTY = 1 << 37;          // "__webpack_require__.o"
    const MAKE_NAMESPACE_OBJECT = 1 << 36;      // "__webpack_require__.r"

    // Import-related
    const REQUIRE = 1 << 5;                     // "__webpack_require__"
    const MODULE = 1 << 3;                      // "module"

    // Advanced features
    const ASYNC_MODULE = 1 << 50;               // async module support
    const HARMONY_MODULE_DECORATOR = 1 << 51;   // ESM compatibility
  }
}
```

**Runtime Code Injection**:

```javascript
// Generated runtime functions (simplified)
__webpack_require__.d = function (exports, definition) {
	for (var key in definition) {
		if (
			__webpack_require__.o(definition, key) &&
			!__webpack_require__.o(exports, key)
		) {
			Object.defineProperty(exports, key, {
				enumerable: true,
				get: definition[key]
			});
		}
	}
};

__webpack_require__.r = function (exports) {
	if (typeof Symbol !== "undefined" && Symbol.toStringTag) {
		Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
	}
	Object.defineProperty(exports, "__esModule", { value: true });
};
```

---

## Tree Shaking Integration

### Usage-Driven Code Generation

```rust
// In ESMExportSpecifierDependencyTemplate::render()
let used_name = ExportsInfoGetter::get_used_name(
  GetUsedNameParam::WithNames(&exports_info),
  runtime,
  std::slice::from_ref(&dep.name),
);

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

### Side Effect Analysis Integration

```rust
fn get_module_evaluation_side_effects_state(
  &self,
  module_graph: &ModuleGraph,
  module_chain: &mut IdentifierSet,
  connection_state_cache: &mut IdentifierMap<ConnectionState>,
) -> ConnectionState {
  // Check for obvious side effects
  let imported_module = module_graph.get_module_by_dependency_id(&self.id)?;

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

### Export Mangling Integration

```rust
// From MangleExportsPlugin
fn mangle_export_name(&self, export_info: &ExportInfo, used_exports: &mut Vec<&str>) -> Option<String> {
  if !export_info.can_mangle_provide().unwrap_or(false)
    || !export_info.can_mangle_use().unwrap_or(false) {
    return None;
  }

  // Generate short name based on frequency or deterministic algorithm
  let mangled_name = if self.deterministic {
    self.generate_deterministic_name(export_info.name()?)
  } else {
    self.generate_frequency_based_name(used_exports)
  };

  Some(mangled_name)
}
```

---

## ConsumeShared and Module Federation

### Tree-Shaking Macro System

Rspack has sophisticated support for Module Federation with fine-grained tree-shaking through conditional macros:

```rust
// Enhanced ConsumeShared processing in ESMExportInitFragment
if s.1.contains("@common:if") && s.1.contains("@common:endif") {
  if let Some(condition_start) = s.1.find("[condition=\"") {
    if let Some(condition_end) = s.1[condition_start..].find("\"]") {
      let condition = &s.1[condition_start..condition_start + condition_end + 2];
      let share_key = extract_share_key_from_condition(condition);

      // Wrap ALL exports for ConsumeShared modules
      let clean_value = s.1
        .cow_replace(&format!("/* @common:if {condition} */ "), "")
        .cow_replace(" /* @common:endif */", "")
        .into_owned();
      let clean_value_func = context.returning_function(&clean_value, "");

      Ok(format!(
        "/* @common:if {condition} */ {prop}: {clean_value_func} /* @common:endif */"
      ))
    }
  }
}
```

### Module Federation Detection

```rust
// ConsumeShared module detection
let consume_shared_info = if let Some(parent_module_id) = module_graph.get_parent_module(&self.id) {
  if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
    if parent_module.module_type() == &ModuleType::ConsumeShared {
      Some(parent_module.get_consume_shared_options())
    } else {
      None
    }
  } else {
    None
  }
};
```

### Macro Format and Processing

**Macro Format**:

```javascript
/* @common:if [condition="treeShake.lodash.map"] */
export_value;
/* @common:endif */
```

**Share Key Extraction**:

```rust
fn extract_share_key_from_condition(condition: &str) -> String {
  if let Some(start) = condition.find("treeShake.") {
    let start_pos = start + 10; // Length of "treeShake."
    let after_treeshake = &condition[start_pos..];
    if let Some(dot_pos) = after_treeshake.find('.') {
      return after_treeshake[..dot_pos].to_string(); // Return share_key
    }
  }
  "unknown".to_string()
}
```

### Object Literal Processing for ConsumeShared

```rust
fn process_consume_shared_object_literal(value: &str, share_key: &str) -> String {
  // Handle patterns like ({ prop1, prop2 }) or { prop1, prop2 }
  let properties: Vec<&str> = obj_content.split(',')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .collect();

  let wrapped_properties: Vec<String> = properties
    .into_iter()
    .map(|prop| {
      // Wrap EVERY property with conditional macros
      format!(
        "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
        share_key, prop.trim(), prop.trim()
      )
    })
    .collect();

  // Reconstruct object with proper formatting
  if value.contains("({") {
    format!(
      "({{\n  {}\n}})",
      wrapped_properties.join(",\n  ")
    )
  } else {
    format!(
      "{{\n  {}\n}}",
      wrapped_properties.join(",\n  ")
    )
  }
}
```

---

## Performance Analysis and Optimizations

### Usage-Driven Code Generation

```rust
// In ESMExportSpecifierDependencyTemplate::render()
let used_name = ExportsInfoGetter::get_used_name(
  GetUsedNameParam::WithNames(&exports_info),
  runtime,
  std::slice::from_ref(&dep.name),
);

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

### Side Effect Analysis Integration

```rust
fn get_module_evaluation_side_effects_state(
  &self,
  module_graph: &ModuleGraph,
  module_chain: &mut IdentifierSet,
  connection_state_cache: &mut IdentifierMap<ConnectionState>,
) -> ConnectionState {
  // Check for obvious side effects
  let imported_module = module_graph.get_module_by_dependency_id(&self.id)?;

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

### Export Mangling Integration

```rust
// From MangleExportsPlugin
fn mangle_export_name(&self, export_info: &ExportInfo, used_exports: &mut Vec<&str>) -> Option<String> {
  if !export_info.can_mangle_provide().unwrap_or(false)
    || !export_info.can_mangle_use().unwrap_or(false) {
    return None;
  }

  // Generate short name based on frequency or deterministic algorithm
  let mangled_name = if self.deterministic {
    self.generate_deterministic_name(export_info.name()?)
  } else {
    self.generate_frequency_based_name(used_exports)
  };

  Some(mangled_name)
}
```

---

## Error Handling and Diagnostics

### Usage Collection Flow

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Runtime Behavior and Generated Code

### Usage Collection Flow

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Plugin System Integration

### Usage Collection Flow

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Advanced Use Cases and Edge Cases

### Usage Collection Flow

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Debugging and Troubleshooting

### Usage Collection Flow

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Best Practices and Patterns

### Usage Collection Flow

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Comparison with Other Bundlers

### Usage Collection Flow

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

### Export Metadata Tracking

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

### Usage State Determination

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Visualizations and Diagrams

### Complete Export Processing Flow

```
┌─────────────┐
│ Source Code │
│ export {...}│
└──────┬──────┘
       │ Parse
       ▼
┌─────────────────────┐    ┌──────────────────┐
│ Export Dependency   │ ─→ │ Module Graph     │
│ Creation            │    │ Registration     │
└─────────────────────┘    └──────────────────┘
       │                           │
       │ Flag Exports              │ Get Export Info
       ▼                           ▼
┌─────────────────────┐    ┌──────────────────┐
│ ExportsInfo         │ ←─ │ ExportInfo       │
│ Population          │    │ Creation         │
└─────────────────────┘    └──────────────────┘
       │                           │
       │ Flag Usage                │ Usage Analysis
       ▼                           ▼
┌─────────────────────┐    ┌──────────────────┐
│ Usage State         │    │ Tree Shaking     │
│ Determination       │ ─→ │ Analysis         │
└─────────────────────┘    └──────────────────┘
       │                           │
       │ Code Generation           │ Template Rendering
       ▼                           ▼
┌─────────────────────┐    ┌──────────────────┐
│ Template System     │ ─→ │ InitFragment     │
│ Processing          │    │ Creation         │
└─────────────────────┘    └──────────────────┘
       │                           │
       │ Fragment Processing       │ Runtime Injection
       ▼                           ▼
┌─────────────────────┐    ┌──────────────────┐
│ Code Composition    │ ─→ │ Final JavaScript │
│ & Runtime Globals   │    │ Bundle           │
└─────────────────────┘    └──────────────────┘
```

### Module Graph Connection Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        MODULE GRAPH                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Module A                    Module B                Module C   │
│  ┌─────────────┐            ┌─────────────┐        ┌──────────┐ │
│  │ entry.js    │            │ utils.js    │        │shared.js │ │
│  │             │            │             │        │          │ │
│  │ import B    │──────────→ │ export util │        │export *  │ │
│  │ export foo  │            │ export log  │        │from './d'│ │
│  │ export {bar}│────┐       │             │        │export bar│ │
│  │ from './c'  │    │       └─────────────┘        └──────────┘ │
│  └─────────────┘    │              │                     ▲     │
│          │          │              │                     │     │
│          │          │              │ ExportsInfo         │     │
│          │          │              ▼                     │     │
│          │          │       ┌─────────────┐              │     │
│          │          │       │ ExportInfo  │              │     │
│          │          │       │ - util:Used │              │     │
│          │          │       │ - log:Unused│              │     │
│          │          │       └─────────────┘              │     │
│          │          │                                    │     │
│          │          └─────── ESMExportImportedSpec ──────┘     │
│          │                  Dependency                        │
│          │                                                    │
│          ▼                                                    │
│   ┌─────────────┐                                             │
│   │ ExportsInfo │                                             │
│   │ - foo: Used │                                             │
│   │ - bar: Used │ ←───────────────────────────────────────────┘
│   │   (reexport)│
│   └─────────────┘
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Dependency Type Inheritance Tree

```
                            Dependency
                                │
                    ┌───────────┼───────────┐
                    │                       │
              ModuleDependency         DependencyCodeGeneration
                    │                       │
          ┌─────────┼─────────┐            │
          │         │         │            │
    ESMImport  ESMExport  CommonJS         │
    Dependencies Dependencies Deps         │
          │         │         │            │
          │         │         └─────┬──────┤
          │         │               │      │
          │    ┌────┼─────┐         │      │
          │    │    │     │         │      │
          │    │    │     │         │      │
          │  ESMExp ESMExp ESMExp    │      │
          │  Spec   ImportSpec Expr  │      │
          │    │    │     │         │      │
          │    └────┼─────┼─────────┼──────┤
          │         │     │         │      │
          └─────────┼─────┼─────────┤      │
                    │     │         │      │
                    Template System │      │
                    │     │         │      │
              ┌─────┼─────┼─────────┼──────┤
              │     │     │         │      │
        ESMExpSpec ESMExpImported    │      │
        Template    SpecTemplate     │      │
              │     │               │      │
              └─────┼───────────────┼──────┤
                    │               │      │
              InitFragment System   │      │
                    │               │      │
              ESMExportInitFragment │      │
                    │               │      │
                    └───────────────┼──────┤
                                    │      │
                          RuntimeGlobals   │
                          Requirements     │
                                    │      │
                            Final Bundle   │
                            Generation     │
                                          │
                                   JavaScript
                                   Output
```

### Re-export Chain Resolution

```
Chain: A.utils → B.utils → C.utils

Module A (app.js)
┌─────────────────────────────────────┐
│ export { utils } from './B'         │ ← ESMExportImportedSpecifierDependency
│                                     │   ├─ request: './B'
│ ┌─ Dependency Analysis ─────────────┤   ├─ export_name: 'utils'
│ │ Target: Module B, export 'utils'  │   └─ mode: NormalReexport
│ │ Terminal: false                   │
│ └───────────────────────────────────┤
└─────────────────────────────────────┘
              │
              ▼ resolve target
Module B (libs.js)
┌─────────────────────────────────────┐
│ export { utils } from './C'         │ ← ESMExportImportedSpecifierDependency
│                                     │   ├─ request: './C'
│ ┌─ Dependency Analysis ─────────────┤   ├─ export_name: 'utils'
│ │ Target: Module C, export 'utils'  │   └─ mode: NormalReexport
│ │ Terminal: false                   │
│ └───────────────────────────────────┤
└─────────────────────────────────────┘
              │
              ▼ resolve target
Module C (shared.js)
┌─────────────────────────────────────┐
│ export const utils = { ... }        │ ← ESMExportSpecifierDependency
│                                     │   ├─ name: 'utils'
│ ┌─ Dependency Analysis ─────────────┤   ├─ local: 'utils'
│ │ Target: Local binding             │   └─ terminal_binding: true
│ │ Terminal: true ★                  │
│ └───────────────────────────────────┤
└─────────────────────────────────────┘

Generated Code:
// Module A
__webpack_require__.d(__webpack_exports__, {
  "utils": function() { return B_module.utils; }
});

// Module B
__webpack_require__.d(__webpack_exports__, {
  "utils": function() { return C_module.utils; }
});

// Module C
const utils = { ... };
__webpack_require__.d(__webpack_exports__, {
  "utils": function() { return utils; }
});
```

### Tree Shaking Decision Tree

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

Usage State Impact:
├─ Used (4) ──────────→ Generate export code
├─ OnlyPropertiesUsed ─→ Generate property access optimizations
├─ Unknown (3) ───────→ Generate export code (safe)
├─ NoInfo (2) ────────→ Generate export code (safe)
└─ Unused (0) ────────→ Remove export / Add comment
```

---

## Summary

Rspack's export system represents a sophisticated and highly optimized approach to handling ES module exports with the following key strengths:

1. **Comprehensive Dependency Modeling**: Every export scenario has a dedicated dependency type with specific optimization strategies

2. **Advanced Tree Shaking**: Usage-driven code generation with support for property-level optimization and inlining

3. **Module Federation Integration**: First-class support for ConsumeShared modules with fine-grained tree-shaking through conditional macros

4. **Performance Optimization**: Extensive caching, efficient module graph representation, and minimal runtime overhead

5. **Standards Compliance**: Full ES module semantics with proper re-export chain resolution and circular dependency handling

The system's multi-layered architecture allows it to handle everything from simple named exports to complex Module Federation scenarios while maintaining optimal bundle sizes through sophisticated tree shaking and dead code elimination.

The integration points between parsing, dependency analysis, export metadata tracking, and code generation create a robust foundation for modern JavaScript bundling requirements.
