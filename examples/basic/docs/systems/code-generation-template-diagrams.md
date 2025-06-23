# Rspack Code Generation & Template Processing Diagrams

## 1. Template System Workflow

```
┌─────────────────┐    ┌─────────────────┐    ┌──────────────────┐
│ Export          │ ─→ │ Template        │ ─→ │ InitFragment     │
│ Dependencies    │    │ Rendering       │    │ Creation         │
└─────────────────┘    └─────────────────┘    └──────────────────┘
         │                       │                       │
         │ metadata               │ add fragments         │ sort/merge
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌──────────────────┐
│ Runtime         │    │ Template        │    │ Generated        │
│ Requirements    │    │ Context         │    │ JavaScript       │
│ Collection      │    │ Processing      │    │ Code             │
└─────────────────┘    └─────────────────┘    └──────────────────┘
```

## 2. InitFragment Creation and Composition

### Fragment Processing Pipeline

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

### Fragment Stage Ordering

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

## 3. Template Context Architecture

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

## 4. Runtime Code Injection Process

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

### Runtime Injection Flow

```
┌─────────────────┐    ┌─────────────────┐    ┌──────────────────┐
│ Fragment        │ ─→ │ Runtime         │ ─→ │ Injected         │
│ Requirements    │    │ Function        │    │ Code             │
│ Analysis        │    │ Selection       │    │ Generation       │
└─────────────────┘    └─────────────────┘    └──────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
   EXPORTS flag          __webpack_require__.d     Property getters
   DEFINE_PROPERTY       __webpack_require__.r     Module markers
   HAS_OWN_PROPERTY      __webpack_require__.o     Ownership checks
```

## 5. Generated Code Examples

### Basic Export Generation

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

### Runtime Function Injection

```javascript
// Generated runtime functions
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

### Tree-Shaking Integration

```javascript
// Re-export chain with tree-shaking
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

## 6. Complete Processing Flow

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

## 7. ESMExportInitFragment Core Implementation

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