# Module to Dependency to Export Relationship Flow

## Overview
This diagram illustrates how modules create export dependencies, which provide metadata through ExportInfo, and ultimately generate JavaScript code.

## Core Relationship Flow

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

## Enhanced Flow with Processing Details

### 1. Module Processing Pipeline

```
Source Module (entry.js)
├── Parse Phase
│   ├── ESM export statement detection
│   ├── Identifier extraction (export names, local variables)
│   └── Span information collection
├── Dependency Creation Phase
│   ├── ESMExportSpecifierDependency creation
│   ├── Module identifier binding: javascript/esm|/path/to/module
│   └── Parent-child relationship establishment
└── Registration Phase
    ├── Module Graph registration
    ├── Dependency ID assignment
    └── Export metadata initialization
```

### 2. Export Dependency Types and Processing

```
Export Dependencies
├── ESMExportSpecifierDependency
│   ├── Input: export { foo, bar as baz }
│   ├── Processing: name/local variable mapping
│   └── Output: __webpack_require__.d() calls
├── ESMExportImportedSpecifierDependency
│   ├── Input: export { x } from './mod'
│   ├── Processing: re-export chain resolution
│   └── Output: dynamic re-export code
├── ESMExportExpressionDependency
│   ├── Input: export default class Helper
│   ├── Processing: default export handling
│   └── Output: __webpack_exports__.default assignment
└── CommonJsExportsDependency
    ├── Input: module.exports = {...}
    ├── Processing: CommonJS compatibility
    └── Output: module.exports bridge code
```

### 3. ExportInfo Metadata System

```
ExportInfo Data Structure
├── Basic Properties
│   ├── name: export identifier
│   ├── used: usage tracking (true/false/OnlyPropertiesUsed)
│   ├── can_mangle: optimization flag
│   └── provided: availability status
├── Advanced Properties
│   ├── terminal_binding: final export flag
│   ├── target: re-export target reference
│   ├── max_target: optimization boundary
│   └── side_effects_only: pure function marking
└── Usage Analysis
    ├── usage_state: Used/Unused/OnlyPropertiesUsed/Unknown/NoInfo
    ├── used_in_runtime: runtime usage tracking
    └── properties_used: property-level tracking
```

### 4. Template and Code Generation Flow

```
Template Processing
├── Dependency Analysis
│   ├── Export mode determination (NormalReexport/DynamicReexport/etc.)
│   ├── Usage state evaluation
│   └── Tree-shaking decision
├── InitFragment Creation
│   ├── Runtime globals injection (__webpack_require__, __webpack_exports__)
│   ├── Export descriptor generation
│   └── Initialization code composition
└── Final Code Assembly
    ├── Fragment concatenation
    ├── Source map generation
    └── Bundle output creation
```

## Real-world Processing Example

### Module: `/lodash-es/escape.js`
```
Processing Flow:
1. Parse: export { escape as default }
2. Create: ESMExportSpecifierDependency(name="default", local="escape")
3. Register: javascript/esm|/path/to/lodash-es/escape.js
4. Generate: __webpack_require__.d(__webpack_exports__, { default: () => escape })
```

### Performance Characteristics
- **Module identifier generation**: O(1) per module
- **Dependency creation**: O(n) where n = number of exports
- **ExportInfo updates**: O(1) per export with hash map lookups
- **Template rendering**: O(m) where m = total dependencies across modules

## Debug Information Integration

From build debug output:
- **988+ ESM import dependencies** processed in typical builds
- **Module identifier format**: `javascript/esm|[absolute_path]`
- **Parent-child relationships** established during dependency resolution
- **Plugin execution** follows systematic pattern with consistent debug output

## Memory and Performance Impact

- **Module Graph growth**: Linear with project size
- **Dependency tracking**: Hash map performance O(1) lookups
- **Export metadata**: Lazy initialization for memory efficiency
- **Code generation**: Streaming approach for large bundles