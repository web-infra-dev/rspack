# Runtime module source reuse implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cover runtime-module source reuse with targeted regression cases, remove the extra render-time runtime-module code generation branch, and ship the change with full build, test, lint, and PR verification.

**Architecture:** Add three focused config cases covering custom `fullHash`, custom `dependentHash`, and runtime-module hook source overrides, then strengthen the existing runtime hash watch cases for built-in `get_full_hash` and `get javascript chunk filename`. After the coverage is in place, simplify JS runtime rendering to always consume the cached runtime-module source that is generated after hashing finishes.

**Tech Stack:** Rust, JavaScript, rspack runtime pipeline, rstest config/watch cases, git/GitHub

---

### Task 1: add regression coverage before changing runtime rendering

**Files:**

- Create: `tests/rspack-test/configCases/runtime/runtime-module-full-hash-source-reuse/*`
- Create: `tests/rspack-test/configCases/runtime/runtime-module-dependent-hash-source-reuse/*`
- Create: `tests/rspack-test/configCases/hooks/runtime-module-full-hash-source-override/*`
- Modify: `tests/rspack-test/watchCases/hashes/runtimeChunkFullHash/test.config.js`
- Modify: `tests/rspack-test/watchCases/hashes/runtimeChunkGetChunkFilename/test.config.js`

- [ ] Add the new config/watch regression cases.
- [ ] Run targeted rstest commands and confirm at least the new source-reuse coverage fails before the fix.

### Task 2: remove duplicate render-time code generation

**Files:**

- Modify: `crates/rspack_plugin_javascript/src/runtime.rs`

- [ ] Update `render_runtime_modules` to always reuse `runtime_modules_code_generation_source`.
- [ ] Re-run the new and strengthened runtime regression cases until they pass.

### Task 3: full verification and publication

**Files:**

- Modify: `docs/superpowers/plans/2026-04-15-runtime-module-source-reuse.md`

- [ ] Run `pnpm run build:cli:dev`.
- [ ] Run the targeted runtime regressions until green.
- [ ] Run `pnpm run test:rs` and `pnpm run test:unit` until green.
- [ ] Run formatting and clippy/lint checks until green.
- [ ] Commit only the intended changes, push the branch, and open a PR.
