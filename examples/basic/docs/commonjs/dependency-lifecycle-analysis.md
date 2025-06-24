# CommonJS Dependencies Lifecycle Analysis

## Overview

This document provides a comprehensive analysis of how CommonJS dependencies are created, processed, and rendered throughout the Rspack build pipeline. Each dependency type handles specific patterns of CommonJS usage, from simple exports to complex re-export scenarios.

## Table of Contents

1. [CommonJsExportsDependency](#commonjsexportsdependency)
2. [CommonJsRequireDependency](#commonjsrequiredependency)
3. [CommonJsExportRequireDependency](#commonjsexportrequiredependency)
4. [CommonJsFullRequireDependency](#commonjsfullrequiredependency)
5. [CommonJsSelfReferenceDependency](#commonjsselfreferencedependency)
6. [Data Flow Diagrams](#data-flow-diagrams)
7. [Integration Architecture](#integration-architecture)

## CommonJsExportsDependency

### Purpose
Handles all CommonJS export patterns including direct property assignments, Object.defineProperty calls, and module.exports assignments.

### Creation Points

**File:** `/crates/rspack_plugin_javascript/src/parser_plugin/common_js_exports_parse_plugin.rs`

#### Assignment Expressions (Lines 442-449)
```rust
// Handles: exports.foo = value, module.exports.bar = value
parser.dependencies.push(Box::new(CommonJsExportsDependency::new(
    left_expr.span().into(),           // Source location
    Some(assign_expr.right.span().into()), // Value range
    base,                             // ExportsBase type
    remaining.to_owned(),             // Property names
)));
```

#### Object.defineProperty Calls (Lines 539-546)
```rust
// Handles: Object.defineProperty(exports, "prop", descriptor)
parser.dependencies.push(Box::new(CommonJsExportsDependency::new(
    call_expr.span.into(),
    Some(arg2.span().into()),
    base,
    vec![str.value.clone()],
)));
```

### Detected Patterns
- `exports.foo = value`
- `module.exports.bar = value`
- `this.baz = value` (top-level context)
- `Object.defineProperty(exports, "prop", descriptor)`
- `Object.defineProperty(module.exports, "prop", descriptor)`

### Template Rendering

**Key Method:** `CommonJsExportsDependencyTemplate::render()`

#### Process Flow
1. **ConsumeShared Detection** - Identifies Module Federation shared contexts
2. **Export Usage Analysis** - Determines which exports are used via `ExportsInfoGetter`
3. **Base Expression Generation** - Converts export base to runtime expression
4. **Conditional Rendering** - Generates tree-shaking macros for shared modules

#### Generated Output Examples

**Standard Export:**
```javascript
// Input:  exports.myFunction = function() { /* ... */ };
// Output: __webpack_exports__.myFunction = function() { /* ... */ };
```

**ConsumeShared with Tree-shaking:**
```javascript
// Input:  exports.myFunction = function() { /* ... */ };
// Output: /* @common:if [condition="treeShake.shared-lib.myFunction"] */ 
//         __webpack_exports__.myFunction = function() { /* ... */ } 
//         /* @common:endif */
```

### Data Flow
```
JavaScript Source
    ↓
CommonJsExportsParserPlugin (AST analysis)
    ↓
Pattern Recognition (assignments, defineProperty)
    ↓
CommonJsExportsDependency::new()
    ↓
Module Graph Integration
    ↓
Template Rendering (with ConsumeShared detection)
    ↓
Final Bundle Output
```

## CommonJsRequireDependency

### Purpose
Handles `require()` calls that import entire modules or are used in expressions.

### Creation Points

**File:** `/crates/rspack_plugin_javascript/src/parser_plugin/common_js_imports_parse_plugin.rs`

#### Require Call Processing (Lines 229-247)
```rust
// Handles: require("./module"), const mod = require("./module")
let dep = CommonJsRequireDependency::new(
    param.string().to_string(),       // Module request
    range,                           // Source location
    None,                           // Expression range
    parser.in_try,                  // Error handling context
    Some(parser.source_map.clone()), // Source mapping
);
```

### Detected Patterns
- `require("./module")`
- `const mod = require("./module")`
- `if (require("./optional-module"))`
- `try { require("./might-fail") } catch {}`

### Template Rendering

**Key Method:** `CommonJsRequireDependencyTemplate::render()`

#### Process Flow
1. **ConsumeShared Detection** - Multi-level algorithm for Module Federation context
2. **Module Resolution** - Maps dependency to actual module via module graph
3. **Code Generation** - Creates `__webpack_require__()` calls
4. **Conditional Macros** - Adds tree-shaking support for shared modules

#### Generated Output Examples

**Standard Require:**
```javascript
// Input:  require("./module")
// Output: __webpack_require__(/*! ./module */ "moduleId")
```

**ConsumeShared Require:**
```javascript
// Input:  require("./shared-module")
// Output: /* @common:if [condition="treeShake.shared-lib.default"] */ 
//         __webpack_require__(/*! ./shared-module */ "moduleId") 
//         /* @common:endif */
```

### Integration Points
- **Module Graph:** Maps dependency ID to resolved module
- **Runtime Template:** Provides `module_id()` function for ID generation
- **ShareUsagePlugin:** Tracks shared module usage for Federation

## CommonJsExportRequireDependency

### Purpose
Handles re-export patterns where modules re-export imports from other modules.

### Creation Points

**File:** `/crates/rspack_plugin_javascript/src/parser_plugin/common_js_exports_parse_plugin.rs`

#### Re-export Detection (Lines 405-414)
```rust
// Handles: module.exports = require('./other'), exports.foo = require('./bar')
parser.dependencies.push(Box::new(CommonJsExportRequireDependency::new(
    param.string().to_string(),       // Source module request
    parser.in_try,                   // Error handling
    range,                          // Source location
    base,                           // Export target (exports/module.exports)
    remaining,                      // Property names
    !parser.is_statement_level_expression(assign_expr.span()), // Result usage
)));
```

### Detected Patterns
- `module.exports = require('./other')` (whole module re-export)
- `module.exports.foo = require('./bar')` (named re-export)
- `exports.baz = require('./utils')` (property re-export)
- `this.api = require('./api')` (context re-export)

### Export Analysis

**Key Method:** `get_exports()` (Lines 214-291)

#### Export Types
1. **Single Named Export** - `module.exports.foo = require('./bar')`
2. **Star Re-export** - `module.exports = require('./other')`
3. **Nested Property Export** - `module.exports.api.client = require('./client')`

### Template Rendering

#### Process Flow
1. **Module Resolution** - Links re-export to source module
2. **Export Usage Analysis** - Determines which re-exports are used
3. **ConsumeShared Integration** - Adds Federation-aware macros
4. **Code Generation** - Creates optimized assignment statements

#### Generated Output Examples

**Whole Module Re-export:**
```javascript
// Input:  module.exports = require('./utils')
// Output: module.exports = __webpack_require__(/*! ./utils */ "moduleId")
```

**Named Re-export with Tree-shaking:**
```javascript
// Input:  module.exports.helper = require('./helper')
// Output: /* @common:if [condition="treeShake.shared-lib.helper"] */ 
//         module.exports.helper = __webpack_require__(/*! ./helper */ "moduleId")
//         /* @common:endif */
```

## CommonJsFullRequireDependency

### Purpose
Handles property access on require calls, enabling tree-shaking optimizations for specific exports.

### Creation Points

**File:** `/crates/rspack_plugin_javascript/src/parser_plugin/common_js_imports_parse_plugin.rs`

#### Property Access Detection (Lines 196-227)
```rust
// Handles: require('./lodash').map, require('./utils').formatDate()
CommonJsFullRequireDependency::new(
    param.string().to_owned(),        // Module request
    members,                         // Property access chain
    range,                          // Source location
    is_call,                        // Whether it's a method call
    parser.in_try,                  // Error context
    !parser.is_asi_position(mem_expr.span_lo()), // ASI safety
    Some(parser.source_map.clone()), // Source mapping
)
```

### Detected Patterns
- `require('./lodash').map` (property access)
- `require('./utils').formatDate()` (method call)
- `require('./api').client.get()` (chained access)
- `require('./helper').createHelper('config')` (factory pattern)

### Template Rendering

**Key Method:** `CommonJsFullRequireDependencyTemplate::render()` (Lines 152-230)

#### Optimization Process
1. **Export Usage Analysis** - Analyzes which exports are actually used
2. **Property Chain Resolution** - Optimizes property access chains
3. **Inlining Opportunities** - Supports direct function inlining
4. **ASI Safety** - Handles automatic semicolon insertion edge cases

#### Generated Output Examples

**Property Access Optimization:**
```javascript
// Input:  require('./lodash').map(array, fn)
// Output: __webpack_require__(/*! ./lodash */ "moduleId").map(array, fn)
```

**Function Inlining:**
```javascript
// Input:  require('./utils').simple()
// Output: /* inlined export simple */ utils_simple_function()
```

### Tree-shaking Integration
- Provides detailed export usage information
- Supports property chain analysis for nested optimizations
- Integrates with Module Federation for cross-bundle optimization

## CommonJsSelfReferenceDependency

### Purpose
Handles self-references within CommonJS modules, preventing circular dependency issues.

### Creation Points

**File:** `/crates/rspack_plugin_javascript/src/parser_plugin/common_js_exports_parse_plugin.rs`

#### Self-Reference Detection
```rust
// Multiple creation scenarios:

// 1. Standalone exports identifier (Lines 289-299)
CommonJsSelfReferenceDependency::new(
    ident.span().into(),
    ExportsBase::Exports,
    vec![],
    false,
);

// 2. Top-level this expression (Lines 310-325)
CommonJsSelfReferenceDependency::new(
    expr.span().into(),
    ExportsBase::This,
    vec![],
    false,
);

// 3. Member expressions (Lines 328-367)
CommonJsSelfReferenceDependency::new(
    expr.span().into(),
    base,                    // Exports, ModuleExports, or This
    remaining,              // Property chain
    false,
);

// 4. Call expressions (Lines 469-566)
CommonJsSelfReferenceDependency::new(
    expr.span().into(),
    base,
    remaining,
    true,                   // is_call = true
);
```

### Detected Patterns
- `exports` (standalone reference)
- `module.exports` (module reference)
- `this` (top-level context)
- `exports.foo.bar` (property chains)
- `module.exports.method()` (method calls)

### Template Rendering

#### Self-Module Architecture
Uses `SelfModuleFactory` to create virtual modules that point back to the original module, preventing infinite recursion in dependency resolution.

#### Generated Output Examples
```javascript
// Input:  module.exports.getSelf = function() { return module.exports; }
// Output: __webpack_exports__.getSelf = function() { return __webpack_exports__; }
```

### Circular Reference Handling
- Creates virtual "self" modules via `SelfModuleFactory`
- Points back to original module (`self {module_identifier}`)
- Integrates with export analysis for tree-shaking compatibility

## Data Flow Diagrams

### Overall CommonJS Processing Pipeline

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   JavaScript    │    │   SWC Parser     │    │      AST        │
│     Source      │───▶│  (Syntax Tree)   │───▶│   Analysis      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
                                                         ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Dependency    │◀───│  Parser Plugins  │◀───│   Pattern       │
│   Creation      │    │   (Visitors)     │    │  Recognition    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │
         ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  Module Graph   │───▶│   Template       │───▶│    Bundle       │
│  Integration    │    │   Rendering      │    │    Output       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Dependency Type Decision Tree

```
require() call?
├─ Yes: Property access?
│  ├─ Yes: CommonJsFullRequireDependency
│  └─ No: CommonJsRequireDependency
└─ No: Export statement?
   ├─ Yes: Assignment to require()?
   │  ├─ Yes: CommonJsExportRequireDependency
   │  └─ No: Self-reference?
   │     ├─ Yes: CommonJsSelfReferenceDependency
   │     └─ No: CommonJsExportsDependency
   └─ No: Continue parsing...
```

### Template Rendering Flow

```
┌─────────────────┐
│   Dependency    │
│   Template      │
│   .render()     │
└─────────┬───────┘
          │
          ▼
┌─────────────────┐    ┌──────────────────┐
│ ConsumeShared   │───▶│   Export Usage   │
│   Detection     │    │    Analysis      │
└─────────────────┘    └─────────┬────────┘
                                 │
                                 ▼
┌─────────────────┐    ┌──────────────────┐
│   Runtime       │◀───│  Code Generation │
│ Requirements    │    │   (with macros)  │
└─────────────────┘    └─────────┬────────┘
                                 │
                                 ▼
                       ┌──────────────────┐
                       │  Source Code     │
                       │   Replacement    │
                       └──────────────────┘
```

## Integration Architecture

### Module Graph Integration

Each dependency type integrates with the module graph system:

1. **Registration** - Dependencies are registered with appropriate factories
2. **Resolution** - Module paths are resolved to actual modules
3. **Analysis** - Export/import relationships are analyzed
4. **Optimization** - Tree-shaking and other optimizations are applied

### Template System Integration

```rust
// Factory Registration Pattern
compilation.set_dependency_factory(
    DependencyType::CjsExports,
    params.normal_module_factory.clone(),
);

// Template Registration Pattern
compilation.set_dependency_template(
    CommonJsExportsDependencyTemplate::template_type(),
    Arc::new(CommonJsExportsDependencyTemplate::default()),
);
```

### Module Federation Integration

All CommonJS dependencies include enhanced ConsumeShared detection:

1. **Multi-level Detection** - Checks parent modules and incoming connections
2. **Macro Generation** - Creates conditional compilation directives
3. **Context Propagation** - Maintains sharing context through dependency chains
4. **Usage Tracking** - Integrates with ShareUsagePlugin for optimization

### Performance Characteristics

- **Parsing Phase** - Dependencies created during AST traversal
- **Resolution Phase** - Module graph construction and validation
- **Optimization Phase** - Export analysis and tree-shaking
- **Generation Phase** - Template rendering and code output

This comprehensive lifecycle analysis shows how Rspack's CommonJS dependency system provides sophisticated module handling while maintaining compatibility with modern JavaScript tooling features like tree-shaking and Module Federation.