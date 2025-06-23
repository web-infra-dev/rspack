# Export Dependency Type Hierarchy

## Overview
This diagram shows the inheritance relationships between different dependency types in Rspack's export system, illustrating how traits and implementations are organized.

## Dependency Type Inheritance Tree

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

## Enhanced Hierarchy with Implementation Details

### Trait-Based Architecture

```
Core Traits
├── Dependency (base trait)
│   ├── dependency_id() -> DependencyId
│   ├── dependency_type() -> &DependencyType
│   ├── category() -> &DependencyCategory
│   ├── span() -> Option<ErrorSpan>
│   └── get_referenced_exports() -> Option<ReferencedExports>
│
├── ModuleDependency (extends Dependency)
│   ├── request() -> &str
│   ├── user_request() -> &str
│   ├── get_condition() -> Option<DependencyCondition>
│   ├── get_referenced_exports() -> Option<ReferencedExports>
│   └── could_affect_referencing_module() -> AffectType
│
└── DependencyCodeGeneration
    ├── generate() -> TWithDependencyInitFragments<CodeGenerationResult>
    ├── generate_exports() -> Option<CodeGenerationResult>
    └── get_attributes() -> Option<Vec<Attribute>>
```

### ESM Export Dependencies Implementation

```
ESM Export Family
├── ESMExportSpecifierDependency
│   ├── Handles: export { foo, bar as baz }
│   ├── Fields:
│   │   ├── span: DependencySpan
│   │   ├── name: Atom (export name)
│   │   ├── local: Atom (local variable)
│   │   └── inline: bool (optimization flag)
│   ├── Template: ESMExportSpecifierDependencyTemplate
│   └── Generated Code: __webpack_require__.d() calls
│
├── ESMExportImportedSpecifierDependency
│   ├── Handles: export { x } from './mod'
│   ├── Fields:
│   │   ├── request: ModuleRequest
│   │   ├── names: Vec<(Atom, Option<Atom>)>
│   │   ├── export_mode: ExportMode
│   │   └── attributes: Option<ImportAttributes>
│   ├── Export Modes:
│   │   ├── Missing: Export not found
│   │   ├── Unused: Tree-shaken out
│   │   ├── ReexportDynamicDefault: Dynamic default re-export
│   │   ├── ReexportNamedDefault: Named default re-export
│   │   ├── NormalReexport: Standard re-export
│   │   └── DynamicReexport: Runtime re-export loop
│   └── Template: ESMExportImportedSpecifierDependencyTemplate
│
└── ESMExportExpressionDependency
    ├── Handles: export default expression
    ├── Fields:
    │   ├── span: DependencySpan
    │   ├── declaration_id: Option<Id>
    │   └── range: Range<u32>
    ├── Template: ESMExportExpressionDependencyTemplate
    └── Generated Code: __webpack_exports__.default assignment
```

### CommonJS Export Dependencies

```
CommonJS Export Family
├── CommonJsExportsDependency
│   ├── Handles: module.exports = {...}
│   ├── Fields:
│   │   ├── range: Range<u32>
│   │   ├── value_range: Option<Range<u32>>
│   │   └── base: Option<ModuleRequest>
│   ├── Template: CommonJsExportsDependencyTemplate
│   └── Generated Code: Module.exports bridge
│
├── CommonJsExportRequireDependency
│   ├── Handles: module.exports = require('./mod')
│   ├── Template: CommonJsExportRequireDependencyTemplate
│   └── Generated Code: Direct module.exports assignment
│
└── CommonJsSelfReferenceDependency
    ├── Handles: exports.foo = bar
    ├── Template: CommonJsSelfReferenceDependencyTemplate
    └── Generated Code: exports property assignment
```

### External Module Dependencies

```
External Module Family
├── ExternalModuleDependency
│   ├── Handles: External library imports
│   ├── Fields:
│   │   ├── request: ModuleRequest
│   │   ├── external_type: ExternalType
│   │   └── optional: bool
│   └── Template: ExternalModuleDependencyTemplate
│
└── ImportMetaDependency
    ├── Handles: import.meta expressions
    ├── Template: ImportMetaDependencyTemplate
    └── Generated Code: Runtime import.meta object
```

## Template System Integration

```
Template Inheritance Hierarchy
├── DependencyTemplate (base trait)
│   ├── apply() -> TWithDependencyInitFragments<()>
│   ├── dependency_id() -> Option<DependencyId>
│   └── update_hash() -> Hash computation
│
├── ModuleDependencyTemplate
│   ├── apply_to_arguments() -> Arguments processing
│   ├── apply_to_module_path() -> Module path resolution
│   └── apply_to_runtime() -> Runtime injection
│
└── Specific Templates
    ├── ESMExportSpecifierDependencyTemplate
    ├── ESMExportImportedSpecifierDependencyTemplate
    ├── ESMExportExpressionDependencyTemplate
    ├── CommonJsExportsDependencyTemplate
    └── ExternalModuleDependencyTemplate
```

## Code Generation Flow

```
Dependency Processing Pipeline
├── Phase 1: Dependency Creation
│   ├── Parser identifies export statements
│   ├── Creates appropriate dependency instance
│   ├── Assigns dependency ID and type
│   └── Registers with module graph
│
├── Phase 2: Template Application
│   ├── Template system processes dependency
│   ├── Determines export mode and optimization
│   ├── Creates InitFragments for runtime code
│   └── Generates JavaScript expressions
│
├── Phase 3: InitFragment Composition
│   ├── RuntimeGlobals injection (__webpack_require__)
│   ├── Export descriptor creation
│   ├── Initialization code assembly
│   └── Source map generation
│
└── Phase 4: Final Code Assembly
    ├── Fragment concatenation
    ├── Bundle optimization
    ├── Tree-shaking application
    └── Output generation
```

## Debug Information Integration

From build analysis:
- **ESM Import Dependencies**: 988+ entries processed
- **Plugin Processing**: Systematic execution pattern
- **Module Identifiers**: `javascript/esm|[path]` format
- **Dependency Resolution**: Parent-child relationship tracking

## Performance Characteristics

- **Trait dispatch**: Zero-cost abstractions in Rust
- **Template application**: O(n) where n = number of dependencies
- **Code generation**: Streaming for memory efficiency
- **Hash computation**: Incremental updates for caching

## Error Handling Integration

- **Span tracking**: Precise error location reporting
- **Dependency validation**: Type-safe dependency creation
- **Template errors**: Code generation failure handling
- **Module resolution**: Missing dependency detection