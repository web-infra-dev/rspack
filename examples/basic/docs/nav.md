# CommonJS Macro Documentation Navigation

**🔗 Gist**: [ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3](https://gist.github.com/ScriptedAlchemy/8de50698b7a21519e423e489d8aeebf3)

## Core Analysis Documents

### 🔍 Issue Analysis

- **[CommonJS Macro Wrapping Issue](#file-commonjs-macro-wrapping-issue-md)** - Critical bug analysis for CommonJS bulk export patterns and ESM considerations
- **[CommonJS Parser Dependency Flow](#file-commonjs-parser-dependency-flow-md)** - Enhanced flow analysis with BuildMeta integration
- **[ESM Parser Dependency Flow](#file-esm-parser-dependency-flow-md)** - Comprehensive ESM dependency processing with enhanced architecture

### 🛠️ Solution Design

- **[CommonJS Macro Solution Design](#file-commonjs-macro-solution-design-md)** - BuildMeta-based universal fix architecture

## Quick Reference

| File                                                                           | Purpose                | Key Focus                                       |
| ------------------------------------------------------------------------------ | ---------------------- | ----------------------------------------------- |
| [commonjs-macro-wrapping-issue.md](#file-commonjs-macro-wrapping-issue-md)     | Problem identification | Bulk export macro conflicts, stacked endif tags |
| [commonjs-parser-dependency-flow.md](#file-commonjs-parser-dependency-flow-md) | System analysis        | Enhanced parser → dependency → template flow    |
| [esm-parser-dependency-flow.md](#file-esm-parser-dependency-flow-md)           | Comparison system      | ESM processing with BuildMeta integration       |
| [commonjs-macro-solution-design.md](#file-commonjs-macro-solution-design-md)   | Implementation guide   | BuildMeta-based universal solution architecture |

## Architecture Summary

### Current Problems (Both Systems)

- **Late ConsumeShared Detection**: Expensive template-time module graph traversal
- **Redundant Operations**: Repeated detection for each dependency
- **System-Specific Issues**: CommonJS range conflicts, ESM fragment coordination needs

### Enhanced Solution: BuildMeta Pattern

> **🔧 Complete Details**: See [BuildMeta Pattern Analysis](commonjs-macro-solution-design.md#codebase-analysis-findings)

**Key Insight**: ConsumeShared context is **module-level metadata**, not dependency-level. The codebase analysis revealed that **BuildMeta is the perfect fit** - it's designed exactly for module-level parser→template metadata passing.

#### Established Rspack Patterns

1. **BuildMeta/BuildInfo Pattern** ✅ - **Perfect fit** for module-level parser→template metadata
2. **FactorizeInfo Pattern** ✅ - Dependency-level factory metadata (not needed for our use case)
3. **AdditionalData Pattern** ⚠️ - Limited usage for specialized cases (overkill for our needs)

#### Our Revised Approach

```rust
// ENHANCED: Extend existing BuildMeta structure
impl BuildMeta {
  // ... existing fields unchanged
  pub esm: bool,
  pub exports_type: BuildMetaExportsType,

  // NEW: ConsumeShared module-level context
  pub consume_shared_context: Option<ConsumeSharedContext>,
  pub bulk_export_coordination: Option<BulkExportCoordination>,
}
```

### Universal Benefits

- **✅ Perfect Pattern Match**: BuildMeta designed exactly for module-level metadata
- **✅ Parser-Phase Detection**: ConsumeShared context computed once, cached automatically
- **✅ Module Coordination**: Handles CommonJS range conflicts + ESM fragment coordination
- **✅ Zero Dependency Changes**: Dependencies remain completely unchanged
- **✅ Universal Solution**: Single approach for both CommonJS and ESM
- **✅ Backwards Compatible**: All existing behavior preserved

## Implementation Strategy

### Phase-Based Rollout

1. **Phase 1**: Extend BuildMeta structure (no behavior change)
2. **Phase 2**: Add parser detection helpers (not called yet)
3. **Phase 3**: Add enhanced template methods (not triggered yet)
4. **Phase 4**: Activate for ConsumeShared modules only

### Risk Assessment: Why This Is Safe

- **Uses Perfect-Fit Architecture**: BuildMeta designed for module-level parser→template metadata
- **Module-Level Fail-Safe**: Falls back to existing logic if ConsumeShared context not present
- **Zero Dependency Impact**: Dependencies unchanged, only BuildMeta extended
- **Easy Rollback**: Stop populating BuildMeta fields to revert

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

_Navigation for Rspack CommonJS macro generation and Module Federation ConsumeShared integration documentation - Updated with BuildMeta-based universal solution architecture._
