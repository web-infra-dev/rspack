# Side Effects Search Results - Complete Summary

## Overview
Comprehensive search for potential unintended side effects across ALL modified files when the `mf_async_startup` experiment is **disabled** (false).

## Search Methodology
1. Examined ALL modified files in the feature branch
2. Analyzed each code path for unconditional execution
3. Traced parameter propagation through function calls
4. Identified variable declarations in wrong scopes
5. Checked for behavior changes in existing code paths
6. Assessed regression risks

## Files Analyzed
- `crates/rspack_plugin_javascript/src/plugin/mod.rs` - 650+ lines, critical issues
- `crates/rspack_plugin_runtime/src/lib.rs` - Critical change to enable_chunk_loading_plugin
- `crates/rspack_plugin_runtime/src/array_push_callback_chunk_format.rs` - Behavior changes
- `crates/rspack_plugin_runtime/src/startup_chunk_dependencies.rs` - Early exit regression
- `crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs` - Mixed behavior
- `crates/rspack_plugin_mf/src/container/embed_federation_runtime_module.rs` - Conditional startup
- `crates/rspack_plugin_mf/src/container/module_federation_runtime_plugin.rs` - Conditional plugin
- `crates/rspack_plugin_web_worker_template/src/lib.rs` - Hardcoding issues
- `crates/rspack/src/builder/builder_context.rs` - Parameter threading
- `crates/rspack_binding_api/src/raw_options/raw_builtins/mod.rs` - Parameter changes
- `crates/rspack/src/builder/mod.rs` - Config impact
- `packages/rspack/src/config/types.ts` - Type definitions
- `packages/rspack/src/config/normalization.ts` - Config normalization
- `crates/rspack_binding_api/src/lib.rs` - API bindings
- `crates/rspack_binding_api/src/raw_options/raw_experiments/mod.rs` - Experiment config
- `crates/rspack_core/src/options/experiments/mod.rs` - Core experiments
- `examples/basic/index.js` - Test configuration
- `examples/basic/rspack.config.cjs` - Test configuration
- `crates/node_binding/napi-binding.d.ts` - Type definitions

## Key Findings Summary

### CRITICAL ISSUES (3)
1. **STARTUP_ENTRYPOINT Hijacking** (js_plugin lines 569-593, 637-641)
   - New code paths execute for ANY STARTUP_ENTRYPOINT
   - Affects non-federation async startup code
   - Silent behavior change

2. **Startup Chunk Dependencies Early Exit** (startup_chunk_dependencies lines 32-37)
   - Unconditional early return skips STARTUP insertion
   - Affects all STARTUP_ENTRYPOINT users
   - Potential runtime crashes

3. **REQUIRE Chunk Loading Behavior Change** (lib.rs lines 47-53)
   - REQUIRE now uses mf_async_startup flag instead of hardcoded false
   - Breaks non-federation synchronous require() patterns
   - Silent breaking change

### HIGH SEVERITY ISSUES (2)
4. **Unconditional Variable Allocation** (js_plugin lines 357-358)
   - Memory overhead on every entry module processing
   - Vectors created but only used conditionally

5. **New STARTUP_ENTRYPOINT Code Path** (js_plugin lines 569-593)
   - NEW code path without proper feature flag protection
   - Similar to Issue #1

### MEDIUM SEVERITY ISSUES (3)
6. **Passive Flag Semantic Change** (array_push lines 155-156)
   - Semantics now tied to unrelated feature flag
   - Confusing for maintainers

7. **Duplicate Runtime Requirements** (multiple plugins)
   - Both js_plugin and federation plugins insert same requirements
   - Fragile coupling between plugins

8. **Configuration Parameter Threading** (various files)
   - New parameter added through multiple signatures
   - All call sites correctly updated (no breaking change)

### LOW SEVERITY ISSUES (1)
9. **Web Worker Comment/Code Mismatch** (web_worker lines 5-6)
   - Comment doesn't match actual implementation
   - Maintenance confusion only

## Code Execution Analysis

### Unconditional Execution (When mf_async_startup=false)
```
✓ Variable declarations (safe)
✓ Loop processing (safe)
❌ Early return in startup_chunk_dependencies (REGRESSION RISK)
✓ Requirement insertion (safe - swaps ON_CHUNKS_LOADED)
✓ Parameter passing (safe)
❌ STARTUP_ENTRYPOINT handling in js_plugin (REGRESSION RISK)
❌ REQUIRE behavior change in lib.rs (REGRESSION RISK)
```

### Conditional Execution (Protected by mf_async_startup=true)
```
✓ Federation async initialization
✓ Promise.all wrapping
✓ Federation async pattern in embed_federation_runtime
✓ Module federation runtime plugin additions
```

### Variable Scope Issues
```
❌ federation_entry_calls (lines 357-358) - allocated unconditionally
❌ all_chunk_ids (lines 357-358) - allocated unconditionally
✓ is_federation_async (line 328) - safe, used for control flow
```

### Parameter Propagation
```
✓ builder_context.rs - correctly passes mf_async_startup
✓ raw_builtins.rs - correctly receives and passes parameter
✓ enable_chunk_loading_plugin - correctly uses parameter
✓ web_worker_template - correctly hardcodes true
```

## Specific Code Issues

See `SPECIFIC_CODE_ISSUES.md` for detailed analysis of all 7 issues with:
- Exact code snippets
- Before/after comparisons
- Real-world scenarios
- Specific fix recommendations
- Test cases needed

## Detailed Analysis

See `SIDE_EFFECTS_ANALYSIS.md` for complete technical analysis including:
- Line-by-line code examination
- Regression risk assessment
- Untested edge cases
- Recommendations by priority

## Quick Reference

See `SIDE_EFFECTS_SUMMARY.txt` for:
- Quick reference table of all issues
- Severity rankings
- Line number references
- Risk assessment matrix

## Test Cases Required

Before merge, these scenarios must be tested:

### Test 1: Non-Federation STARTUP_ENTRYPOINT
- Verify STARTUP_ENTRYPOINT works without mf_async_startup
- Check NO federation wrapping occurs
- Verify STARTUP is added when needed

### Test 2: REQUIRE Chunk Loading
- Verify synchronous behavior with mf_async_startup=true
- Check no unexpected async chunk loading
- Test Node.js require() patterns

### Test 3: Array Push Callback
- Verify passive mode entry startup
- Check no semantic issues with passive flag inversion
- Test entry module initialization

### Test 4: Startup Dependencies
- Verify STARTUP added when STARTUP_ENTRYPOINT present
- Check no early exit for non-federation code
- Test chunk dependency loading

## Recommendations

### MUST FIX Before Merge:
1. Add mf_async_startup check to STARTUP_ENTRYPOINT conditions
2. Add mf_async_startup check to startup_chunk_dependencies early exit
3. Revert REQUIRE behavior or add federation-specific context

### SHOULD FIX:
4. Move variable allocation inside conditional block
5. Add regression tests for all scenarios

### NICE TO FIX:
6. Add comments explaining passive flag inversion
7. Document plugin coupling and coordination
8. Fix web worker comment

## Overall Risk Assessment

**Current Status**: NOT READY FOR PRODUCTION
- 3 critical issues affecting non-federation code
- 2 high severity issues
- Potential regressions in existing builds
- Silent failures (no error messages)

**If Fixed**: SAFE TO MERGE
- Proper feature isolation
- No impact on non-federation code
- Good foundation for async federation

## Document Index

1. **SIDE_EFFECTS_FINAL_REPORT.txt** - Executive summary and full analysis
2. **SPECIFIC_CODE_ISSUES.md** - 7 specific issues with exact code patterns
3. **SIDE_EFFECTS_ANALYSIS.md** - Complete technical analysis
4. **SIDE_EFFECTS_SUMMARY.txt** - Quick reference table
5. **SEARCH_RESULTS_SUMMARY.md** - This document

All analysis files are located in `/Users/zackjackson/rspack/`

## Next Steps

1. Review the critical issues identified
2. Implement fixes as outlined
3. Add regression tests
4. Re-run analysis to verify fixes
5. Test in staging environment before production merge

---

**Analysis completed**: October 27, 2025
**Branch analyzed**: feature/async-startup-runtime-promise
**Total modified files**: 18
**Total issues identified**: 9 (3 critical, 2 high, 3 medium, 1 low)
**Estimated effort to fix**: 4-6 hours development + testing
