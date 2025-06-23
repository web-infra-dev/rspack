# Rspack Module Federation Visual Flows

## Overview
This document provides comprehensive visual flows showing how the Rspack Module Federation sharing system processes imports, determines exports, and generates tree-shaking annotations.

## Complete Import-to-Resolution Flow

### Main Application Import Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    MAIN APPLICATION                             │
│                                                                 │
│   src/app.js:                                                  │
│   import { debounce, map } from 'lodash';                      │
│   import { Component } from 'react';                           │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │                MODULE RESOLUTION                            │ │
│ │                                                             │ │
│ │  1. Factorize Hook                                          │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ ConsumeSharedPlugin.factorize()                 │   │ │
│ │     │ ├─ Check "lodash" in unresolved patterns        │   │ │
│ │     │ ├─ Check "react" in unresolved patterns         │   │ │
│ │     │ └─ Create ConsumeSharedModule instances         │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │  2. Module Creation                                         │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ ConsumeSharedModule {                           │   │ │
│ │     │   share_key: "lodash",                          │   │ │
│ │     │   share_scope: "default",                       │   │ │
│ │     │   fallback: "lodash",                           │   │ │
│ │     │   version: "^4.17.0",                           │   │ │
│ │     │   singleton: true,                              │   │ │
│ │     │   eager: false                                  │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    DEPENDENCY RESOLUTION                        │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │                ConsumeSharedModule.build()                  │ │
│ │                                                             │ │
│ │  1. Fallback Resolution                                     │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ import: "lodash"                                │   │ │
│ │     │ ├─ Resolver.resolve()                           │   │ │
│ │     │ ├─ Result: "./node_modules/lodash/index.js"     │   │ │
│ │     │ └─ Create: ConsumeSharedFallbackDependency      │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │  2. Loading Strategy                                        │ │
│ │     ┌─────────────────┐  ┌─────────────────────────────┐   │ │
│ │     │ Eager Loading   │  │ Lazy Loading                │   │ │
│ │     │ ├─ Direct Dep   │  │ ├─ AsyncDependenciesBlock   │   │ │
│ │     │ └─ Sync Access  │  │ └─ Promise-based Loading    │   │ │
│ │     └─────────────────┘  └─────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    EXPORT METADATA PROCESSING                   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │            finish_modules Hook Execution                    │ │
│ │                                                             │ │
│ │  copy_exports_from_fallback_to_consume_shared()             │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ Source: ./node_modules/lodash/index.js          │   │ │
│ │     │ ├─ ExportsInfo.get_provided_exports()           │   │ │
│ │     │ ├─ Result: ["debounce", "map", "filter", ...]   │   │ │
│ │     │ └─ Export metadata for each function            │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ Target: ConsumeSharedModule("lodash")           │   │ │
│ │     │ ├─ Copy export names and capabilities           │   │ │
│ │     │ ├─ Copy mangle information                      │   │ │
│ │     │ ├─ Copy nested export info                      │   │ │
│ │     │ └─ Set has_provide_info = true                  │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Export Determination Process

### Fallback Module Analysis

```
┌─────────────────────────────────────────────────────────────────┐
│                 FALLBACK MODULE ANALYSIS                        │
│                                                                 │
│ Input: ConsumeSharedModule("lodash")                           │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │              Find Fallback Module                           │ │
│ │                                                             │ │
│ │  1. API Method (Preferred)                                  │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ ConsumeSharedModule::find_fallback_module_id()  │   │ │
│ │     │ └─ Returns: ModuleIdentifier                    │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │  2. Dependency Traversal (Fallback)                        │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ for dep in module.get_dependencies() {          │   │ │
│ │     │   if dep.dependency_type() ==                   │   │ │
│ │     │      "consume shared fallback" {                │   │ │
│ │     │     return module_graph                         │   │ │
│ │     │       .module_identifier_by_dependency_id()     │   │ │
│ │     │   }                                             │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │              Analyze Fallback Exports                       │ │
│ │                                                             │ │
│ │  Result: ./node_modules/lodash/index.js                    │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ ExportsInfoGetter::prefetch()                   │   │ │
│ │     │ ├─ Mode: PrefetchExportsInfoMode::AllExports    │   │ │
│ │     │ ├─ Provided: ["debounce", "map", "filter",      │   │ │
│ │     │ │            "throttle", "chunk", "compact",    │   │ │
│ │     │ │            "concat", "difference", ...]       │   │ │
│ │     │ ├─ Side Effects: false                          │   │ │
│ │     │ └─ Export Details: {                            │   │ │
│ │     │     "debounce": {                               │   │ │
│ │     │       provided: true,                           │   │ │
│ │     │       can_mangle: true,                         │   │ │
│ │     │       terminal_binding: true                    │   │ │
│ │     │     }                                           │   │ │
│ │     │   }                                             │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Import Usage Detection

```
┌─────────────────────────────────────────────────────────────────┐
│                    IMPORT USAGE DETECTION                       │
│                                                                 │
│ Target: ConsumeSharedModule("lodash")                          │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │           Analyze Incoming Connections                      │ │
│ │                                                             │ │
│ │  1. Get All Incoming Connections                            │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ connections = module_graph                       │   │ │
│ │     │   .get_incoming_connections(consume_shared_id)   │   │ │
│ │     │                                                 │   │ │
│ │     │ Example connections:                            │   │ │
│ │     │ ├─ From: src/app.js                             │   │ │
│ │     │ │  Dependency: ESMImportSpecifierDependency     │   │ │
│ │     │ │  Imports: ["debounce", "map"]                 │   │ │
│ │     │ └─ From: src/utils.js                           │   │ │
│ │     │    Dependency: ESMImportSpecifierDependency     │   │ │
│ │     │    Imports: ["debounce"]                        │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │  2. Filter Active Connections                               │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ for connection in connections {                 │   │ │
│ │     │   match connection.active_state() {             │   │ │
│ │     │     Active(true) => process,                    │   │ │
│ │     │     TransitiveOnly => process,                  │   │ │
│ │     │     CircularConnection => skip                  │   │ │
│ │     │   }                                             │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │  3. Extract Referenced Exports                             │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ dependency.get_referenced_exports()             │   │ │
│ │     │                                                 │   │ │
│ │     │ Results:                                        │   │ │
│ │     │ ├─ ExtendedReferencedExport::Array([           │   │ │
│ │     │ │    "debounce", "map"                          │   │ │
│ │     │ │  ])                                           │   │ │
│ │     │ └─ Usage Analysis:                              │   │ │
│ │     │    ├─ Imported: ["debounce", "map"]            │   │ │
│ │     │    └─ Actually Used: ["debounce"]               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Tree-Shaking Analysis Flow

### Usage State Determination

```
┌─────────────────────────────────────────────────────────────────┐
│                  USAGE STATE DETERMINATION                      │
│                                                                 │
│ ConsumeShared Module: "lodash"                                 │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │                Export Classification                        │ │
│ │                                                             │ │
│ │  Input Data:                                                │ │
│ │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │ │
│ │  │ Provided        │  │ Imported        │  │ Actually    │ │ │
│ │  │ Exports         │  │ Exports         │  │ Used        │ │ │
│ │  │                 │  │                 │  │ Exports     │ │ │
│ │  │ • debounce      │  │ • debounce      │  │ • debounce  │ │ │
│ │  │ • map           │  │ • map           │  │             │ │ │
│ │  │ • filter        │  │ • filter        │  │             │ │ │
│ │  │ • throttle      │  │                 │  │             │ │ │
│ │  │ • chunk         │  │                 │  │             │ │ │
│ │  │ • ... (200+)    │  │                 │  │             │ │ │
│ │  └─────────────────┘  └─────────────────┘  └─────────────┘ │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │              Classification Algorithm                       │ │
│ │                                                             │ │
│ │  For each export in provided_exports:                       │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ if export in actually_used_exports:             │   │ │
│ │     │   classification = "Used"                       │   │ │
│ │     │ else if export in imported_exports:             │   │ │
│ │     │   classification = "ImportedButUnused"          │   │ │
│ │     │ else:                                           │   │ │
│ │     │   classification = "NotImported"                │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                                                             │ │
│ │  Results:                                                   │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ "debounce": "Used"                              │   │ │
│ │     │ "map": "ImportedButUnused"                      │   │ │
│ │     │ "filter": "ImportedButUnused"                   │   │ │
│ │     │ "throttle": "NotImported"                       │   │ │
│ │     │ "chunk": "NotImported"                          │   │ │
│ │     │ ... (remaining 200+ exports): "NotImported"     │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Tree-Shaking Annotation Generation

```
┌─────────────────────────────────────────────────────────────────┐
│                 TREE-SHAKING ANNOTATION GENERATION              │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │              Export Usage Detail Creation                   │ │
│ │                                                             │ │
│ │  For each classified export:                                │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ ExportUsageDetail {                             │   │ │
│ │     │   export_name: "debounce",                      │   │ │
│ │     │   usage_state: "Used",                          │   │ │
│ │     │   can_mangle: Some(true),                       │   │ │
│ │     │   can_inline: Some(false),                      │   │ │
│ │     │   is_provided: Some(true),                      │   │ │
│ │     │   used_name: Some("debounce"),                  │   │ │
│ │     │   annotation: "KEEP"                            │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ ExportUsageDetail {                             │   │ │
│ │     │   export_name: "map",                           │   │ │
│ │     │   usage_state: "ImportedButUnused",             │   │ │
│ │     │   can_mangle: Some(true),                       │   │ │
│ │     │   can_inline: Some(true),                       │   │ │
│ │     │   is_provided: Some(true),                      │   │ │
│ │     │   used_name: None,                              │   │ │
│ │     │   annotation: "ELIMINATE"                       │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │               Report Generation                             │ │
│ │                                                             │ │
│ │  ShareUsageReport {                                         │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ consume_shared_modules: {                       │   │ │
│ │     │   "lodash": ShareUsageData {                    │   │ │
│ │     │     used_exports: ["debounce"],                 │   │ │
│ │     │     unused_imports: ["map", "filter"],          │   │ │
│ │     │     provided_exports: ["debounce", "map", ...], │   │ │
│ │     │     export_details: [/* detailed analysis */],  │   │ │
│ │     │     has_unused_imports: true,                   │   │ │
│ │     │     fallback_info: { /* fallback analysis */ } │   │ │
│ │     │   }                                             │   │ │
│ │     │ },                                              │   │ │
│ │     │ analysis_metadata: { /* timing, cache stats */ },│   │ │
│ │     │ diagnostics: [],                                │   │ │
│ │     │ performance_metrics: { /* profiling data */ }   │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │  }                                                          │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Pure Annotation System Flow

### ConsumeShared Descendant Detection

```
┌─────────────────────────────────────────────────────────────────┐
│               PURE ANNOTATION DETECTION FLOW                    │
│                                                                 │
│ Import Statement: import { debounce } from 'lodash';           │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │             Dependency Type Check                           │ │
│ │                                                             │ │
│ │  1. Check Import Type                                       │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ dep_type = dependency.dependency_type()         │   │ │
│ │     │                                                 │   │ │
│ │     │ if dep_type == "esm import specifier" &&        │   │ │
│ │     │    import_var != "__webpack_require__" {        │   │ │
│ │     │   // This is a named/default import             │   │ │
│ │     │   is_named_import = true                        │   │ │
│ │     │ } else {                                        │   │ │
│ │     │   // This is a side-effect import               │   │ │
│ │     │   is_named_import = false                       │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │  2. ConsumeShared Descendant Check                          │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ if is_named_import {                            │   │ │
│ │     │   is_consume_shared_descendant(                 │   │ │
│ │     │     module_graph,                               │   │ │
│ │     │     current_module.identifier()                 │   │ │
│ │     │   )                                             │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │          Recursive Ancestor Search                          │ │
│ │                                                             │ │
│ │  is_consume_shared_descendant_recursive(                    │ │
│ │    module_graph, current_module, visited, max_depth        │ │
│ │  ) {                                                        │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ // Check current module                         │   │ │
│ │     │ if module.module_type() == ConsumeShared {      │   │ │
│ │     │   return true                                   │   │ │
│ │     │ }                                               │   │ │
│ │     │                                                 │   │ │
│ │     │ // Check incoming connections                   │   │ │
│ │     │ for connection in get_incoming_connections() {  │   │ │
│ │     │   if origin_module.module_type() ==            │   │ │
│ │     │      ConsumeShared {                           │   │ │
│ │     │     return true                                 │   │ │
│ │     │   }                                             │   │ │
│ │     │   // Recurse up to max_depth = 10              │   │ │
│ │     │   if recursive_check(origin_module) {           │   │ │
│ │     │     return true                                 │   │ │
│ │     │   }                                             │   │ │
│ │     │ }                                               │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │  }                                                          │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │              Generated Code Output                          │ │
│ │                                                             │ │
│ │  Pure Annotation Applied:                                   │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ // Input: import { debounce } from 'lodash';    │   │ │
│ │     │ // Output (ConsumeShared descendant):           │   │ │
│ │     │ /* ESM import */var lodash = /* #__PURE__ */    │   │ │
│ │     │   __webpack_require__("lodash");                │   │ │
│ │     │                                                 │   │ │
│ │     │ // Input: import './styles.css';                │   │ │
│ │     │ // Output (side-effect import):                 │   │ │
│ │     │ __webpack_require__("./styles.css");            │   │ │
│ │     │ // (no pure annotation)                         │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Runtime Code Generation Flow

### ConsumeShared Runtime Module Generation

```
┌─────────────────────────────────────────────────────────────────┐
│                RUNTIME CODE GENERATION FLOW                     │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │           Code Generation Phase                             │ │
│ │                                                             │ │
│ │  ConsumeSharedModule.code_generation()                      │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ 1. Determine Loader Function                    │   │ │
│ │     │    ├─ Base: "loaders.load"                      │   │ │
│ │     │    ├─ + Version: "loadVersionCheck"             │   │ │
│ │     │    ├─ + Strict: "loadStrictVersionCheck"        │   │ │
│ │     │    └─ + Singleton: "loadSingletonVersionCheck"  │   │ │
│ │     │                                                 │   │ │
│ │     │ 2. Generate Arguments                           │   │ │
│ │     │    ├─ shareScope: "default"                     │   │ │
│ │     │    ├─ shareKey: "lodash"                        │   │ │
│ │     │    ├─ version: "^4.17.0"                        │   │ │
│ │     │    └─ fallback: factory function                │   │ │
│ │     │                                                 │   │ │
│ │     │ 3. Create Factory Function                      │   │ │
│ │     │    ├─ Eager: sync_module_factory()              │   │ │
│ │     │    └─ Lazy: async_module_factory()              │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ │                    │                                        │ │
│ │                    ▼                                        │ │
│ │  CodeGenerationDataConsumeShared {                          │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ share_scope: "default",                         │   │ │
│ │     │ share_key: "lodash",                            │   │ │
│ │     │ import: false,                                  │   │ │
│ │     │ required_version: Some("^4.17.0"),              │   │ │
│ │     │ strict_version: false,                          │   │ │
│ │     │ singleton: true,                                │   │ │
│ │     │ eager: false,                                   │   │ │
│ │     │ fallback: Some("function() { ... }")           │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │          Runtime Module Generation                          │ │
│ │                                                             │ │
│ │  ConsumeSharedRuntimeModule.generate()                      │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ 1. Collect Module Data                          │   │ │
│ │     │    ├─ Iterate all chunks                        │   │ │
│ │     │    ├─ Find ConsumeShared modules                │   │ │
│ │     │    └─ Extract CodeGenerationData                │   │ │
│ │     │                                                 │   │ │
│ │     │ 2. Build Mappings                               │   │ │
│ │     │    ├─ chunkMapping: chunk → module IDs          │   │ │
│ │     │    ├─ moduleIdToConsumeDataMapping: config      │   │ │
│ │     │    └─ initialConsumes: eager modules            │   │ │
│ │     │                                                 │   │ │
│ │     │ 3. Generate JavaScript Code                     │   │ │
│ │     │    ├─ __webpack_require__.consumesLoadingData   │   │ │
│ │     │    ├─ Include JavaScript loaders                │   │ │
│ │     │    └─ Include loader selection logic            │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                    │                                            │
│                    ▼                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │              Final JavaScript Output                        │ │
│ │                                                             │ │
│ │     ┌─────────────────────────────────────────────────┐   │ │
│ │     │ __webpack_require__.consumesLoadingData = {     │   │ │
│ │     │   chunkMapping: {                               │   │ │
│ │     │     "main": ["default-lodash"]                  │   │ │
│ │     │   },                                            │   │ │
│ │     │   moduleIdToConsumeDataMapping: {               │   │ │
│ │     │     "lodash": {                                 │   │ │
│ │     │       shareScope: "default",                   │   │ │
│ │     │       shareKey: "lodash",                       │   │ │
│ │     │       requiredVersion: "^4.17.0",               │   │ │
│ │     │       singleton: true,                          │   │ │
│ │     │       fallback: function() { ... }             │   │ │
│ │     │     }                                           │   │ │
│ │     │   },                                            │   │ │
│ │     │   initialConsumes: []                           │   │ │
│ │     │ };                                              │   │ │
│ │     │                                                 │   │ │
│ │     │ // JavaScript loaders for version resolution   │   │ │
│ │     │ var load = ...                                  │   │ │
│ │     │ var loadVersionCheck = ...                      │   │ │
│ │     │ var resolveHandler = ...                        │   │ │
│ │     └─────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Summary: Complete Basic Example Flow

### From Import Statement to Tree-Shaking

```
Application Code:
┌─────────────────────────────────────────────────────────────────┐
│ // src/app.js                                                  │
│ import { debounce, map } from 'lodash';                        │
│ import { Component } from 'react';                             │
│                                                                 │
│ const debouncedFn = debounce(myFunction, 300);                 │
│ // Note: 'map' is imported but never used                      │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
Module Federation Processing:
┌─────────────────────────────────────────────────────────────────┐
│ 1. ConsumeSharedPlugin creates ConsumeSharedModule("lodash")    │
│ 2. Fallback resolved to "./node_modules/lodash/index.js"       │
│ 3. Export metadata copied from fallback to ConsumeShared       │
│ 4. Usage analysis detects: imported=["debounce","map"]         │
│                            used=["debounce"]                   │
│ 5. Tree-shaking marks "map" as unused import                   │
│ 6. Pure annotations applied to ConsumeShared descendants       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
Generated Runtime Code:
┌─────────────────────────────────────────────────────────────────┐
│ // Pure annotated import for tree-shaking                      │
│ /* ESM import */var lodash = /* #__PURE__ */                   │
│   __webpack_require__("lodash");                               │
│                                                                 │
│ // Runtime loading with fallback                               │
│ __webpack_require__.consumesLoadingData = {                    │
│   moduleIdToConsumeDataMapping: {                              │
│     "lodash": {                                                │
│       shareScope: "default",                                  │
│       shareKey: "lodash",                                      │
│       singleton: true,                                         │
│       fallback: function() {                                  │
│         return __webpack_require__("./node_modules/lodash");   │
│       }                                                        │
│     }                                                          │
│   }                                                            │
│ };                                                             │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
Tree-Shaking Analysis Output:
┌─────────────────────────────────────────────────────────────────┐
│ {                                                               │
│   "lodash": {                                                  │
│     "used_exports": ["debounce"],                              │
│     "unused_imports": ["map"],                                 │
│     "provided_exports": ["debounce", "map", "filter", ...],    │
│     "has_unused_imports": true,                                │
│     "export_details": [                                        │
│       {                                                        │
│         "export_name": "debounce",                             │
│         "usage_state": "Used",                                 │
│         "annotation": "KEEP"                                   │
│       },                                                       │
│       {                                                        │
│         "export_name": "map",                                  │
│         "usage_state": "ImportedButUnused",                    │
│         "annotation": "ELIMINATE"                              │
│       }                                                        │
│     ]                                                          │
│   }                                                            │
│ }                                                              │
└─────────────────────────────────────────────────────────────────┘
```

This comprehensive visual flow demonstrates how Rspack's Module Federation sharing system processes imports from initial request through to tree-shaking analysis, providing detailed insights for optimizing shared module usage in micro-frontend architectures.