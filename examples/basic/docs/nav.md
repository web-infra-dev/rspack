# CommonJS Macro Documentation Navigation

**🔗 Gist**: [ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3](https://gist.github.com/ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3)

## Core Analysis Documents

### 🔍 Issue Analysis

- **[CommonJS Macro Wrapping Issue](#file-commonjs-macro-wrapping-issue-md)** - Critical bug analysis for CommonJS bulk export patterns and ESM considerations
- **[CommonJS Parser Dependency Flow](#file-commonjs-parser-dependency-flow-md)** - Enhanced flow analysis with BuildMeta integration
- **[ESM Parser Dependency Flow](#file-esm-parser-dependency-flow-md)** - Comprehensive ESM dependency processing with enhanced architecture
- **[Actual Rspack Implementation Analysis](actual-rspack-implementation-analysis.md)** - Real codebase findings: actual hooks, data structures, async patterns

### 🛠️ Solution Design

- **[CommonJS Macro Solution Design](#file-commonjs-macro-solution-design-md)** - BuildMeta-based universal fix architecture

## Quick Reference

| File                                                                           | Purpose                | Key Focus                                       |
| ------------------------------------------------------------------------------ | ---------------------- | ----------------------------------------------- |
| [commonjs-macro-wrapping-issue.md](#file-commonjs-macro-wrapping-issue-md)     | Problem identification | Bulk export macro conflicts, stacked endif tags |
| [commonjs-parser-dependency-flow.md](#file-commonjs-parser-dependency-flow-md) | System analysis        | Enhanced parser → dependency → template flow    |
| [esm-parser-dependency-flow.md](#file-esm-parser-dependency-flow-md)           | Comparison system      | ESM processing with BuildMeta integration       |
| [commonjs-macro-solution-design.md](#file-commonjs-macro-solution-design-md)   | Implementation guide   | BuildMeta-based universal solution architecture |
| [actual-rspack-implementation-analysis.md](actual-rspack-implementation-analysis.md) | Real codebase analysis | Actual hooks, data structures, memory layout, async patterns |

## Architecture Summary

### Current Problems (Both Systems)

- **Late ConsumeShared Detection**: Expensive template-time module graph traversal
- **Redundant Operations**: Repeated detection for each dependency
- **System-Specific Issues**: CommonJS range conflicts, ESM fragment coordination needs

### Enhanced Solution: ConsumeSharedPlugin Extension + BuildMeta Pattern

> **🔧 Complete Details**: See [NormalModuleFactory + BuildMeta Architecture](commonjs-macro-solution-design.md#revised-solution-architecture-normalmodulefactory--buildmeta-pattern)
> **🚀 Super Massive Flow**: See [Rust Plugin Technical Flow](comprehensive-system-flow-analysis.md#super-massive-rust-plugin-source-code-technical-flow)

**Key Insight**: **ConsumeSharedPlugin already exists** with perfect infrastructure. The optimal solution **extends existing patterns** rather than creating new plugins. Codebase analysis revealed the **perfect integration approach**:

#### 🎯 Optimal Integration Strategy (Leverage Existing)

1. **Extend ConsumeSharedPlugin** ✅ - **BEST: Use existing NormalModuleFactory hooks**
   - Plugin already hooks NormalModuleFactoryFactorize (lines 657-741)
   - Add NormalModuleFactoryAfterResolve for BuildMeta population
   - Leverage existing MatchedConsumes logic (lines 715-741)

2. **BuildMeta/BuildInfo Pattern** ✅ - **Perfect metadata storage**
   - Module-level parser→template metadata passing
   - Automatic caching and serialization
   - Established Rspack pattern (24 existing fields)

3. **Parser Coordination Enhancement** ✅ - **Minimal changes to existing bulk export handling**
   - Use pre-computed BuildMeta context
   - Coordinate range management for bulk exports

#### 🏗️ Three-Tier Architecture Implementation

```rust
// TIER 1: Early Detection (Extend Existing ConsumeSharedPlugin)
impl ConsumeSharedPlugin {
  #[plugin_hook(NormalModuleFactoryAfterResolve)]
  async fn after_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
    // Use existing detection logic from lines 715-741
    if let Some(config) = self.find_consume_config(&data.request) {
      // Set BuildMeta BEFORE parsing begins (prevents all conflicts)
      data.build_info.build_meta.consume_shared_key = Some(config.share_key.clone());
    }
  }
}

// TIER 2: BuildMeta Storage (Extend Existing Pattern)
impl BuildMeta {
  // ... existing 24 fields unchanged
  pub consume_shared_key: Option<String>,           // NEW: Early detection result
  pub export_coordination: Option<ExportCoordination>, // NEW: Parser coordination data
}

// TIER 3: Parser Enhancement (Use Pre-computed Context)
impl CommonJsExportsParserPlugin {
  fn handle_bulk_assignment(&mut self, parser: &mut JavascriptParser) {
    // Use existing BuildMeta context - NO detection needed
    if let Some(share_key) = &parser.build_meta.consume_shared_key {
      parser.build_meta.export_coordination = Some(ExportCoordination::CommonJS { ... });
    }
  }
}
```

### Universal Benefits

- **✅ Leverages Existing Infrastructure**: ConsumeSharedPlugin already has perfect patterns
- **✅ Early Detection**: ConsumeShared context set before parsing prevents all conflicts
- **✅ Module Coordination**: Handles CommonJS range conflicts + ESM fragment coordination
- **✅ Zero Dependency Changes**: Dependencies remain completely unchanged
- **✅ Performance Optimization**: Eliminates expensive template-time module graph traversal
- **✅ Backwards Compatible**: All existing behavior preserved
- **✅ Minimal Code Changes**: Extends existing patterns rather than creating new systems

## Implementation Strategy

### Phase-Based Rollout

1. **Phase 1**: Critical cleanup - remove architectural violations (FlagDependencyUsagePlugin changes)
2. **Phase 2**: Extend BuildMeta structure + ConsumeSharedPlugin hook (no behavior change)
3. **Phase 3**: Add parser coordination using pre-computed BuildMeta context
4. **Phase 4**: Simplify template logic to use BuildMeta instead of module graph traversal
5. **Phase 5**: Activate coordinated macro generation for ConsumeShared modules

### Risk Assessment: Why This Is Safe

- **Leverages Existing Architecture**: ConsumeSharedPlugin + BuildMeta are established patterns
- **Early Detection Prevents Conflicts**: BuildMeta set before parsing eliminates range conflicts
- **Module-Level Fail-Safe**: Falls back to existing logic if ConsumeShared context not present
- **Zero Dependency Impact**: Dependencies unchanged, only BuildMeta metadata extended
- **Easy Rollback**: Stop populating BuildMeta fields to revert to current behavior

## Raw File Links

For direct access to file contents:

- [commonjs-macro-wrapping-issue.md](https://gist.githubusercontent.com/ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3/raw/commonjs-macro-wrapping-issue.md)
- [commonjs-parser-dependency-flow.md](https://gist.githubusercontent.com/ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3/raw/commonjs-parser-dependency-flow.md)
- [esm-parser-dependency-flow.md](https://gist.githubusercontent.com/ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3/raw/esm-parser-dependency-flow.md)
- [commonjs-macro-solution-design.md](https://gist.githubusercontent.com/ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3/raw/commonjs-macro-solution-design.md)

## Related Documentation

### Subdirectories

- **[commonjs/](./commonjs/)** - Detailed CommonJS implementation analysis
- **[systems/](./systems/)** - System architecture and dependency hierarchies

---

_Navigation for Rspack CommonJS macro generation and Module Federation ConsumeShared integration documentation - Updated with ConsumeSharedPlugin extension + BuildMeta pattern solution architecture._
