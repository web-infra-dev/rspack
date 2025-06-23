# Rspack Module Graph Architecture Patterns

## Core Module Graph Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                     MODULE GRAPH CORE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐    ┌──────────────────┐    ┌─────────────┐ │
│  │ ModuleGraph     │───→│ ModuleGraphModule│───→│ Connections │ │
│  │                 │    │                  │    │             │ │
│  │ - modules       │    │ - dependencies   │    │ - source    │ │
│  │ - connections   │    │ - exports_info   │    │ - target    │ │
│  │ - exports_info  │    │ - incoming_conn  │    │ - dependency│ │
│  └─────────────────┘    └──────────────────┘    └─────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Module Interconnection Patterns

```
┌─────────────────────────────────────────────────────────────────┐
│                   MODULE INTERCONNECTIONS                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Module A (entry.js)     Module B (utils.js)    Module C       │
│  ┌─────────────┐        ┌─────────────┐        ┌──────────┐    │
│  │             │        │             │        │shared.js │    │
│  │ import B    │────────│ export util │        │          │    │
│  │ export foo  │        │ export log  │        │export *  │    │
│  │ export {bar}│────┐   │             │        │from './d'│    │
│  │ from './c'  │    │   └─────────────┘        │export bar│    │
│  └─────────────┘    │                          └──────────┘    │
│          │          │                               ▲          │
│          │ creates  │ creates                       │          │
│          ▼          ▼                               │          │
│   ┌─────────────┐  ESMExportImportedSpec ──────────┘          │
│   │ ExportsInfo │  Dependency (re-export chain)               │
│   │ - foo: Used │                                             │
│   │ - bar: Used │                                             │
│   │   (reexport)│                                             │
│   └─────────────┘                                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Dependency Network Flow

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

## ExportsInfo Relationship Mapping

```
┌─────────────────────────────────────────────────────────────────┐
│                 EXPORTSINFO RELATIONSHIPS                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐           ┌─────────────┐           ┌─────────┐ │
│  │ ExportsInfo │◇─────────→│ ExportInfo  │◇─────────→│ Target  │ │
│  │             │  contains │             │  points   │ Module  │ │
│  │ - other_exp │  (1..*)   │ - name      │  to       │         │ │
│  │ - redirect  │           │ - used_name │  (0..1)   │         │ │
│  │ - side_eff  │           │ - provided  │           │         │ │
│  └─────────────┘           │ - can_mangle│           └─────────┘ │
│         │                  │ - terminal  │                      │
│         │ redirects_to     │ - used_state│                      │
│         ▼                  └─────────────┘                      │
│  ┌─────────────┐                  │                             │
│  │ ExportsInfo │                  │ nested                      │
│  │ (target)    │                  ▼                             │
│  └─────────────┘           ┌─────────────┐                      │
│                            │ ExportsInfo │                      │
│                            │ (nested)    │                      │
│                            └─────────────┘                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Graph Traversal Patterns

```
┌─────────────────────────────────────────────────────────────────┐
│                   TRAVERSAL PATTERNS                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. DEPENDENCY RESOLUTION                                       │
│     Module A ──dep──→ Module B ──exports──→ ExportsInfo        │
│                                                                 │
│  2. RE-EXPORT CHAIN TRAVERSAL                                  │
│     Module A ──reexp──→ Module B ──reexp──→ Module C           │
│        │                   │                   │               │
│        └─── ExportInfo ────┴─── ExportInfo ────┘               │
│             (redirect)          (terminal)                     │
│                                                                 │
│  3. USAGE STATE PROPAGATION                                    │
│     Consumer ──uses──→ ExportInfo ──affects──→ Provider        │
│         │                 │                       │            │
│         └── UsageState ───┴── UsageState ────────┘            │
│                                                                 │
│  4. TREE SHAKING ELIMINATION                                   │
│     ExportInfo.unused ──eliminates──→ Dead Code Removal        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Usage State Flow Patterns

```
┌────────────────────────────────────────────────────────────────┐
│                    USAGE STATE HIERARCHY                      │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│        Unused (0) ──┐                                         │
│                     ├─→ OnlyPropertiesUsed (1) ──┐            │
│                     ├─→ NoInfo (2) ──────────────┼─→ Unknown  │
│                     └─→ Used (4) ←───────────────┴─→ (3) ──┐  │
│                                                            │   │
│    ┌───────────────────────────────────────────────────────┘   │
│    ▼                                                           │
│  FINAL USAGE DETERMINATION                                     │
│    │                                                          │
│    ├─→ Tree Shake (Unused/OnlyPropertiesUsed)                │
│    ├─→ Keep & Optimize (Used with mangle info)               │
│    └─→ Keep Conservative (Unknown/NoInfo)                    │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

## Processing Flow Architecture

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

## Key Architectural Insights

### Connection Patterns
- **Direct Dependencies**: Module A imports from Module B
- **Re-export Chains**: Module A exports from Module C via Module B
- **Circular Dependencies**: Handled through ModuleGraphConnection tracking

### Metadata Relationships
- **ExportsInfo**: Container for all export metadata of a module
- **ExportInfo**: Individual export tracking with usage, targeting, and mangling info
- **ModuleGraphConnection**: Links modules through dependencies with optimization hints

### Graph Traversal Optimization
- **Terminal Binding**: Stops traversal at final export providers
- **Usage Propagation**: Flows backward from consumers to providers
- **Tree Shaking**: Forward elimination based on usage analysis