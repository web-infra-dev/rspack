# Flag Dependency Exports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a two-phase `FlagDependencyExportsPlugin` flow that collects dependency exports in parallel, defers non-nested reexport resolution into SCC wave propagation, and proves the new code is faster than latest `origin/main`.

**Architecture:** Add a recoverable `DependencyExportsAnalysisArtifact` in `FINISH_MODULES` to store per-module local export operations, deferred reexport edges, SCC membership, and propagation waves. Refactor `FlagDependencyExportsPlugin` into collector, local-apply, and propagation stages so local structure is applied first and deferred reexports are resolved wave-by-wave, with fixed-point iteration only inside a single SCC.

**Tech Stack:** Rust (`rspack_core`, `rspack_plugin_javascript`), Rayon, rspack incremental artifacts, pnpm `rspack-test`, `cargo codspeed`

---

## File Map

### Core artifact and compilation plumbing

- Modify: `crates/rspack_core/src/dependency/mod.rs`
  - Extend `ExportsSpec` with explicit processing metadata for deferred reexports.
- Create: `crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs`
  - Store per-module collected analysis, topology cache, dirty flags, and unit tests.
- Modify: `crates/rspack_core/src/artifacts/mod.rs`
  - Export the new artifact.
- Modify: `crates/rspack_core/src/compilation/mod.rs`
  - Add the artifact to `Compilation` and extend the `CompilationFinishModules` hook signature.
- Modify: `crates/rspack_core/src/compilation/finish_modules/mod.rs`
  - Steal/restore the artifact during `finish_modules`.
- Modify: `crates/rspack_core/src/cache/memory.rs`
  - Recover the new artifact before `finish_modules`.

### FlagDependencyExportsPlugin refactor

- Modify: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs`
  - Keep hook orchestration and wire new helper modules.
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/types.rs`
  - Shared collector/apply/propagation types.
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/collector.rs`
  - Parallel collection and normalization into `LocalCollected` vs `DeferredReexport`.
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/local_apply.rs`
  - Flat parallel local apply and structured sequential local apply.
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/propagation.rs`
  - SCC wave scheduling and SCC-local fixed-point propagation.

### Reexport emitters

- Modify: `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_imported_specifier_dependency.rs`
  - Emit deferred metadata for non-nested reexports, keep nested/fake-namespace cases local.
- Modify: `crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_export_require_dependency.rs`
  - Emit deferred metadata for property/whole-module reexports that should participate in topology propagation.

### Tests

- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/index.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/root.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/left.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/right.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/leaf-left.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/leaf-right.js`

### Existing end-to-end regression cases to rerun

- Reuse: `tests/rspack-test/normalCases/esm/reexport-with-cycle`
- Reuse: `tests/rspack-test/normalCases/parsing/esm-commonjs-interop`
- Reuse: `tests/rspack-test/normalCases/side-effects/dynamic-reexports`
- Reuse: `tests/rspack-test/normalCases/json/reexport`

---

### Task 1: Capture The Baseline Before Touching Code

**Files:**
- Modify: none
- Test: none

- [ ] **Step 1: Create a detached baseline worktree at latest `origin/main`**

Run:

```bash
git fetch origin --prune
git worktree add --detach /Users/bytedance/.config/superpowers/worktrees/rspack/flag-dependency-exports-baseline origin/main
git -C /Users/bytedance/.config/superpowers/worktrees/rspack/flag-dependency-exports-baseline rev-parse --short HEAD
```

Expected: the last command prints the same SHA as `origin/main`.

- [ ] **Step 2: Prepare benchmark inputs in the baseline worktree**

Run:

```bash
cd /Users/bytedance/.config/superpowers/worktrees/rspack/flag-dependency-exports-baseline
pnpm run bench:prepare
```

Expected: `scripts/bench/setup.mjs` completes and `.bench/rspack-benchcases` exists.

- [ ] **Step 3: Build the CodSpeed benchmark binary for the baseline**

Run:

```bash
cd /Users/bytedance/.config/superpowers/worktrees/rspack/flag-dependency-exports-baseline
cargo codspeed build -m simulation --profile codspeed -p rspack_benchmark --features codspeed
```

Expected: the command exits `0` and finishes without changing tracked source files.

- [ ] **Step 4: Record baseline CodSpeed output**

Run:

```bash
cd /Users/bytedance/.config/superpowers/worktrees/rspack/flag-dependency-exports-baseline
env RAYON_NUM_THREADS=1 pnpm run bench:rust | tee /tmp/flag-dependency-exports-baseline.txt
```

Expected: `/tmp/flag-dependency-exports-baseline.txt` contains the benchmark summary for latest `origin/main`.

---

### Task 2: Add `DependencyExportsAnalysisArtifact` And `finish_modules` Plumbing

**Files:**
- Create: `crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs`
- Modify: `crates/rspack_core/src/artifacts/mod.rs`
- Modify: `crates/rspack_core/src/compilation/mod.rs`
- Modify: `crates/rspack_core/src/compilation/finish_modules/mod.rs`
- Modify: `crates/rspack_core/src/cache/memory.rs`
- Test: `crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs`

- [ ] **Step 1: Write the failing artifact recovery tests**

Add this test module at the bottom of `crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::incremental::{Incremental, IncrementalOptions, IncrementalPasses};
  use crate::ModuleIdentifier;

  #[test]
  fn recover_keeps_previous_finish_modules_state_and_marks_it_dirty() {
    let module = ModuleIdentifier::from("module-a");
    let mut old = DependencyExportsAnalysisArtifact::default();
    old.replace_module(module, ModuleDependencyExportsAnalysis::default());
    old.set_topology_dirty(false);

    let incremental = Incremental::new_hot(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::FINISH_MODULES,
    });

    let mut new = DependencyExportsAnalysisArtifact::default();
    DependencyExportsAnalysisArtifact::recover(&incremental, &mut new, &mut old);

    assert!(new.modules().contains_key(&module));
    assert!(new.topology_dirty());
  }
}
```

- [ ] **Step 2: Run the new test and verify it fails**

Run:

```bash
cargo test -p rspack_core recover_keeps_previous_finish_modules_state_and_marks_it_dirty -- --nocapture
```

Expected: FAIL with unresolved items such as `DependencyExportsAnalysisArtifact`, `replace_module`, or `topology_dirty`.

- [ ] **Step 3: Implement the artifact, compilation field, hook plumbing, and memory-cache recovery**

Add the new artifact and wire it through the finish-modules lifecycle.

`crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs`:

```rust
#[derive(Debug, Default, Clone)]
pub struct DependencyExportsAnalysisArtifact {
  modules: IdentifierMap<ModuleDependencyExportsAnalysis>,
  topology: DependencyExportsTopology,
  topology_dirty: bool,
}

impl ArtifactExt for DependencyExportsAnalysisArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::FINISH_MODULES;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if incremental.mutations_readable(Self::PASS) {
      std::mem::swap(new, old);
      new.mark_all_dirty();
      new.set_topology_dirty(true);
    }
  }
}
```

`crates/rspack_core/src/compilation/mod.rs`:

```rust
define_hook!(CompilationFinishModules: Series(
  compilation: &Compilation,
  async_modules_artifact: &mut AsyncModulesArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact
));

pub dependency_exports_analysis_artifact: StealCell<DependencyExportsAnalysisArtifact>,
```

`crates/rspack_core/src/compilation/finish_modules/mod.rs`:

```rust
let mut dependency_exports_analysis_artifact =
  compilation.dependency_exports_analysis_artifact.steal();

compilation
  .plugin_driver
  .clone()
  .compilation_hooks
  .finish_modules
  .call(
    compilation,
    async_modules_artifact,
    exports_info_artifact,
    &mut dependency_exports_analysis_artifact,
  )
  .await?;

compilation.dependency_exports_analysis_artifact =
  dependency_exports_analysis_artifact.into();
```

`crates/rspack_core/src/cache/memory.rs`:

```rust
recover_artifact(
  incremental,
  &mut compilation.dependency_exports_analysis_artifact,
  &mut old_compilation.dependency_exports_analysis_artifact,
);
```

- [ ] **Step 4: Run the focused artifact test and a core/plugin compile check**

Run:

```bash
cargo test -p rspack_core recover_keeps_previous_finish_modules_state_and_marks_it_dirty -- --nocapture
cargo check -p rspack_core -p rspack_plugin_javascript
```

Expected: PASS for the test, then `Finished` or `Checking` output with exit code `0`.

- [ ] **Step 5: Commit the plumbing**

Run:

```bash
git add \
  crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs \
  crates/rspack_core/src/artifacts/mod.rs \
  crates/rspack_core/src/compilation/mod.rs \
  crates/rspack_core/src/compilation/finish_modules/mod.rs \
  crates/rspack_core/src/cache/memory.rs
git commit -m "refactor(core): add dependency exports analysis artifact"
```

---

### Task 3: Build SCC And Wave Scheduling As A Purely Testable Core Primitive

**Files:**
- Modify: `crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs`
- Test: `crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs`

- [ ] **Step 1: Write failing tests for SCC compression and wave grouping**

Add these tests:

```rust
#[test]
fn rebuild_topology_groups_independent_sccs_into_the_same_wave() {
  let mut artifact = DependencyExportsAnalysisArtifact::default();
  let a = ModuleIdentifier::from("a");
  let b = ModuleIdentifier::from("b");
  let c = ModuleIdentifier::from("c");
  let d = ModuleIdentifier::from("d");

  artifact.replace_module(a, ModuleDependencyExportsAnalysis::with_targets([c]));
  artifact.replace_module(b, ModuleDependencyExportsAnalysis::with_targets([d]));
  artifact.replace_module(c, ModuleDependencyExportsAnalysis::default());
  artifact.replace_module(d, ModuleDependencyExportsAnalysis::default());

  artifact.rebuild_topology();

  assert_eq!(artifact.topology().waves().len(), 2);
  assert_eq!(artifact.topology().waves()[0].len(), 2);
}

#[test]
fn rebuild_topology_puts_a_cycle_into_one_scc() {
  let mut artifact = DependencyExportsAnalysisArtifact::default();
  let a = ModuleIdentifier::from("cycle-a");
  let b = ModuleIdentifier::from("cycle-b");

  artifact.replace_module(a, ModuleDependencyExportsAnalysis::with_targets([b]));
  artifact.replace_module(b, ModuleDependencyExportsAnalysis::with_targets([a]));

  artifact.rebuild_topology();

  assert_eq!(artifact.topology().scc_nodes().len(), 1);
}
```

- [ ] **Step 2: Run the topology tests and verify they fail**

Run:

```bash
cargo test -p rspack_core rebuild_topology_ -- --nocapture
```

Expected: FAIL because `with_targets`, `rebuild_topology`, `waves`, or `scc_nodes` do not exist yet.

- [ ] **Step 3: Implement SCC condensation and reverse-topological wave building**

Add focused topology helpers inside the artifact module:

```rust
impl DependencyExportsAnalysisArtifact {
  pub fn rebuild_topology(&mut self) {
    self.topology = DependencyExportsTopology::from_modules(&self.modules);
    self.topology_dirty = false;
  }
}

impl DependencyExportsTopology {
  fn from_modules(
    modules: &IdentifierMap<ModuleDependencyExportsAnalysis>,
  ) -> DependencyExportsTopology {
    let scc = tarjan_scc(modules);
    let condensed = condense_scc_graph(modules, &scc);
    let reverse_topo = reverse_topological_order(&condensed);
    let waves = build_parallel_waves(&condensed, &reverse_topo);
    DependencyExportsTopology {
      module_to_scc: scc.module_to_scc,
      scc_nodes: condensed.nodes,
      waves,
    }
  }
}
```

- [ ] **Step 4: Run the topology tests again**

Run:

```bash
cargo test -p rspack_core rebuild_topology_ -- --nocapture
```

Expected: PASS for both tests.

- [ ] **Step 5: Commit the topology builder**

Run:

```bash
git add crates/rspack_core/src/artifacts/dependency_exports_analysis_artifact.rs
git commit -m "refactor(core): add dependency exports topology waves"
```

---

### Task 4: Extend `ExportsSpec` With Explicit Deferred-Reexport Metadata

**Files:**
- Modify: `crates/rspack_core/src/dependency/mod.rs`
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/types.rs`
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/collector.rs`
- Modify: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs`
- Test: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/collector.rs`

- [ ] **Step 1: Write the failing collector normalization test**

Add this test in `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/collector.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use rspack_core::{
    DeferredReexportItem, DeferredReexportSpec, DependencyId, ExportsOfExportsSpec, ExportsProcessing,
    ExportsSpec, ModuleIdentifier, Nullable,
  };

  #[test]
  fn normalize_exports_spec_keeps_deferred_reexports_out_of_local_apply() {
    let owner = ModuleIdentifier::from("entry");
    let target = ModuleIdentifier::from("leaf");
    let spec = ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![]),
      processing: ExportsProcessing::DeferredReexport(vec![DeferredReexportSpec {
        target_module: target,
        dep_id: DependencyId::from(7),
        items: vec![DeferredReexportItem {
          exposed_name: "value".into(),
          target_path: Nullable::Value(vec!["value".into()]),
          hidden: false,
        }],
        ..Default::default()
      }]),
      ..Default::default()
    };

    let normalized = normalize_exports_spec(owner, spec);
    assert!(normalized.local_apply.is_empty());
    assert_eq!(normalized.deferred_reexports.len(), 1);
  }
}
```

- [ ] **Step 2: Run the collector test and verify it fails**

Run:

```bash
cargo test -p rspack_plugin_javascript normalize_exports_spec_keeps_deferred_reexports_out_of_local_apply -- --nocapture
```

Expected: FAIL with unresolved names such as `DeferredReexportSpec`, `ExportsProcessing`, or `normalize_exports_spec`.

- [ ] **Step 3: Add the shared processing metadata in `rspack_core` and the plugin-side normalizer**

`crates/rspack_core/src/dependency/mod.rs`:

```rust
#[derive(Debug, Clone, Default)]
pub enum ExportsProcessing {
  #[default]
  Immediate,
  DeferredReexport(Vec<DeferredReexportSpec>),
}

#[derive(Debug, Clone, Default)]
pub struct DeferredReexportSpec {
  pub target_module: ModuleIdentifier,
  pub dep_id: DependencyId,
  pub priority: Option<u8>,
  pub can_mangle: Option<bool>,
  pub terminal_binding: bool,
  pub items: Vec<DeferredReexportItem>,
}

#[derive(Debug, Clone)]
pub struct DeferredReexportItem {
  pub exposed_name: Atom,
  pub target_path: Nullable<Vec<Atom>>,
  pub hidden: bool,
}

pub struct ExportsSpec {
  pub exports: ExportsOfExportsSpec,
  pub processing: ExportsProcessing,
  pub priority: Option<u8>,
  pub can_mangle: Option<bool>,
  pub terminal_binding: Option<bool>,
  pub from: Option<ModuleGraphConnection>,
  pub dependencies: Option<Vec<ModuleIdentifier>>,
  pub hide_export: Option<FxHashSet<Atom>>,
  pub exclude_exports: Option<FxHashSet<Atom>>,
}
```

`crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/collector.rs`:

```rust
pub fn normalize_exports_spec(
  owner_module: ModuleIdentifier,
  spec: ExportsSpec,
) -> NormalizedModuleAnalysis {
  match spec.processing {
    ExportsProcessing::Immediate => NormalizedModuleAnalysis::from_local(owner_module, spec),
    ExportsProcessing::DeferredReexport(deferred) => {
      NormalizedModuleAnalysis::from_deferred(owner_module, spec, deferred)
    }
  }
}
```

- [ ] **Step 4: Run the collector test and a package compile check**

Run:

```bash
cargo test -p rspack_plugin_javascript normalize_exports_spec_keeps_deferred_reexports_out_of_local_apply -- --nocapture
cargo check -p rspack_core -p rspack_plugin_javascript
```

Expected: PASS for the collector test, then exit code `0` for `cargo check`.

- [ ] **Step 5: Commit the explicit processing metadata**

Run:

```bash
git add \
  crates/rspack_core/src/dependency/mod.rs \
  crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs \
  crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/types.rs \
  crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/collector.rs
git commit -m "refactor(exports): classify deferred reexports explicitly"
```

---

### Task 5: Teach Reexport-Producing Dependencies To Emit Deferred Metadata

**Files:**
- Modify: `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_imported_specifier_dependency.rs`
- Modify: `crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_export_require_dependency.rs`
- Test: `tests/rspack-test/normalCases/esm/reexport-with-cycle`
- Test: `tests/rspack-test/normalCases/side-effects/dynamic-reexports`
- Test: `tests/rspack-test/normalCases/parsing/esm-commonjs-interop`

- [ ] **Step 1: Update the ESM emitter so only non-nested reexports become deferred**

Change the non-nested reexport branches to set `processing: ExportsProcessing::DeferredReexport`, while keeping fake-namespace and nested cases local:

```rust
ExportMode::NormalReexport(mode) => {
  let from = mg.connection_by_dependency_id(self.id());
  Some(ExportsSpec {
    exports: ExportsOfExportsSpec::Names(vec![]),
    processing: ExportsProcessing::DeferredReexport(vec![DeferredReexportSpec {
      target_module: *from.expect("should have module").module_identifier(),
      dep_id: self.id(),
      priority: Some(1),
      items: mode
        .items
        .into_iter()
        .map(|item| DeferredReexportItem {
          exposed_name: item.name,
          target_path: Nullable::Value(item.ids),
          hidden: item.hidden,
        })
        .collect(),
      ..Default::default()
    }]),
    dependencies: Some(vec![*from.expect("should have module").module_identifier()]),
    ..Default::default()
  })
}
```

- [ ] **Step 2: Update the CommonJS emitter so property and whole-module reexports can be deferred**

Apply the same processing split in `common_js_export_require_dependency.rs`:

```rust
Some(ExportsSpec {
  exports: ExportsOfExportsSpec::Names(vec![]),
  processing: ExportsProcessing::DeferredReexport(vec![DeferredReexportSpec {
    target_module: *from.module_identifier(),
    dep_id: self.id,
    can_mangle: Some(!self.is_all_exported_by_module_exports()),
    items: vec![DeferredReexportItem {
      exposed_name: name.to_owned(),
      target_path: if ids.is_empty() {
        Nullable::Null
      } else {
        Nullable::Value(ids.to_vec())
      },
      hidden: false,
    }],
    ..Default::default()
  }]),
  dependencies: Some(vec![*from.module_identifier()]),
  ..Default::default()
})
```

- [ ] **Step 3: Run targeted regression tests immediately after changing emitters**

Run:

```bash
cd tests/rspack-test
pnpm run test -t "normalCases/esm/reexport-with-cycle|normalCases/side-effects/dynamic-reexports|normalCases/parsing/esm-commonjs-interop"
```

Expected: FAIL at first if the plugin is not yet consuming deferred metadata, or PASS for the interop case while the reexport cases start surfacing the missing propagation wiring.

- [ ] **Step 4: Fix any compile-only breakage from the new emitter shape**

Run:

```bash
cargo check -p rspack_plugin_javascript
```

Expected: exit code `0` so the next task can focus only on the plugin refactor.

- [ ] **Step 5: Commit the emitter changes**

Run:

```bash
git add \
  crates/rspack_plugin_javascript/src/dependency/esm/esm_export_imported_specifier_dependency.rs \
  crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_export_require_dependency.rs
git commit -m "refactor(javascript): emit deferred reexport specs"
```

---

### Task 6: Refactor `FlagDependencyExportsPlugin` Into Collector, Local Apply, And Propagation Stages

**Files:**
- Modify: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs`
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/local_apply.rs`
- Create: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/propagation.rs`
- Test: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/propagation.rs`

- [ ] **Step 1: Write the failing propagation-wave test**

Add this test in `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/propagation.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use rspack_core::{DependencyExportsAnalysisArtifact, ModuleIdentifier};

  #[test]
  fn propagate_runs_independent_sccs_in_the_same_wave_and_converges_per_scc() {
    let mut artifact = DependencyExportsAnalysisArtifact::default();
    let left = ModuleIdentifier::from("left");
    let right = ModuleIdentifier::from("right");
    let root = ModuleIdentifier::from("root");

    artifact.replace_module(left, ModuleDependencyExportsAnalysis::with_targets([]));
    artifact.replace_module(right, ModuleDependencyExportsAnalysis::with_targets([]));
    artifact.replace_module(root, ModuleDependencyExportsAnalysis::with_targets([left, right]));
    artifact.rebuild_topology();

    let summary = propagation_waves(&artifact);
    assert_eq!(summary[0].len(), 2);
    assert_eq!(summary[1].len(), 1);
  }
}
```

- [ ] **Step 2: Run the propagation test and verify it fails**

Run:

```bash
cargo test -p rspack_plugin_javascript propagate_runs_independent_sccs_in_the_same_wave_and_converges_per_scc -- --nocapture
```

Expected: FAIL because `propagation_waves` and the staged plugin helpers do not exist yet.

- [ ] **Step 3: Split the plugin into staged helpers and wire the new finish-modules flow**

Add module declarations in `flag_dependency_exports_plugin.rs` and replace the old batch-backtracking loop with staged calls:

```rust
mod collector;
mod local_apply;
mod propagation;
mod types;

let affected_modules = collect_affected_modules(compilation);
prepare_provide_info(module_graph, exports_info_artifact, &affected_modules);
collector::collect_module_analysis(
  module_graph,
  module_graph_cache,
  exports_info_artifact,
  dependency_exports_analysis_artifact,
  &affected_modules,
)?;
local_apply::apply_local_exports(
  module_graph,
  exports_info_artifact,
  dependency_exports_analysis_artifact,
  &affected_modules,
)?;
propagation::propagate_deferred_reexports(
  module_graph,
  exports_info_artifact,
  dependency_exports_analysis_artifact,
)?;
```

`local_apply.rs` should keep the current split:

```rust
pub fn apply_local_exports(
  mg: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  analysis_artifact: &DependencyExportsAnalysisArtifact,
  affected_modules: &IdentifierSet,
) -> Result<()> {
  apply_flat_local_exports_in_parallel(
    mg,
    exports_info_artifact,
    analysis_artifact,
    affected_modules,
  )?;
  apply_structured_local_exports_sequentially(
    mg,
    exports_info_artifact,
    analysis_artifact,
    affected_modules,
  )?;
  Ok(())
}
```

`propagation.rs` should use wave scheduling:

```rust
for wave in artifact.topology().waves() {
  wave.par_iter().try_for_each(|scc_id| {
    resolve_scc_until_fixed_point(*scc_id, module_graph, exports_info_artifact, artifact)
  })?;
}
```

- [ ] **Step 4: Run the propagation test and the focused plugin test set**

Run:

```bash
cargo test -p rspack_plugin_javascript propagate_runs_independent_sccs_in_the_same_wave_and_converges_per_scc -- --nocapture
cd tests/rspack-test
pnpm run test -t "normalCases/esm/reexport-with-cycle|normalCases/side-effects/dynamic-reexports|normalCases/parsing/esm-commonjs-interop|normalCases/json/reexport"
```

Expected: PASS for the unit test, then PASS for the targeted end-to-end cases.

- [ ] **Step 5: Commit the staged plugin refactor**

Run:

```bash
git add \
  crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs \
  crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/local_apply.rs \
  crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin/propagation.rs
git commit -m "refactor(javascript): stage dependency export propagation"
```

---

### Task 7: Add One New End-To-End Regression For Sibling-Wave Reexports

**Files:**
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/index.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/root.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/left.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/right.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/leaf-left.js`
- Create: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/leaf-right.js`
- Test: `tests/rspack-test/normalCases/esm/reexport-wave-parallel/index.js`

- [ ] **Step 1: Write the new failing `rspack-test` case**

Create `tests/rspack-test/normalCases/esm/reexport-wave-parallel/index.js`:

```javascript
import { leftValue, rightValue } from "./root";

it("should keep sibling reexport waves stable", () => {
  expect(leftValue).toBe("left");
  expect(rightValue).toBe("right");
});
```

Create `tests/rspack-test/normalCases/esm/reexport-wave-parallel/root.js`:

```javascript
export { leftValue } from "./left";
export { rightValue } from "./right";
```

Create `tests/rspack-test/normalCases/esm/reexport-wave-parallel/left.js`:

```javascript
export { leftValue } from "./leaf-left";
```

Create `tests/rspack-test/normalCases/esm/reexport-wave-parallel/right.js`:

```javascript
export { rightValue } from "./leaf-right";
```

Create `tests/rspack-test/normalCases/esm/reexport-wave-parallel/leaf-left.js`:

```javascript
export const leftValue = "left";
```

Create `tests/rspack-test/normalCases/esm/reexport-wave-parallel/leaf-right.js`:

```javascript
export const rightValue = "right";
```

- [ ] **Step 2: Run only the new case and verify it fails if the implementation is incomplete**

Run:

```bash
cd tests/rspack-test
pnpm run test -t "normalCases/esm/reexport-wave-parallel"
```

Expected: FAIL before the staged propagation task is complete, then PASS after the plugin refactor is in place.

- [ ] **Step 3: Run the new case together with the existing high-risk reexport cases**

Run:

```bash
cd tests/rspack-test
pnpm run test -t "normalCases/esm/reexport-wave-parallel|normalCases/esm/reexport-with-cycle|normalCases/parsing/esm-commonjs-interop|normalCases/side-effects/dynamic-reexports|normalCases/json/reexport"
```

Expected: PASS for all listed cases.

- [ ] **Step 4: Commit the regression coverage**

Run:

```bash
git add tests/rspack-test/normalCases/esm/reexport-wave-parallel
git commit -m "test(esm): cover wave-parallel reexport propagation"
```

---

### Task 8: Run Full Validation And Compare CodSpeed Against The Baseline

**Files:**
- Modify: only if a final performance-only cleanup is needed
- Test: full repository validation commands

- [ ] **Step 1: Build the Rust and JS code before running broad tests**

Run:

```bash
pnpm run build:cli:dev
```

Expected: exit code `0` and fresh dev bindings plus JS output.

- [ ] **Step 2: Run the required preflight checks**

Run:

```bash
cargo fmt --all --check
cargo lint
```

Expected: both commands exit `0`.

- [ ] **Step 3: Run the required functional test suites**

Run:

```bash
pnpm run test:rs
pnpm run test:unit
```

Expected: both commands exit `0`.

- [ ] **Step 4: Build and run CodSpeed on the feature branch**

Run:

```bash
pnpm run bench:prepare
cargo codspeed build -m simulation --profile codspeed -p rspack_benchmark --features codspeed
env RAYON_NUM_THREADS=1 pnpm run bench:rust | tee /tmp/flag-dependency-exports-feature.txt
```

Expected: `/tmp/flag-dependency-exports-feature.txt` contains the feature-branch benchmark summary.

- [ ] **Step 5: Compare the feature results to the `origin/main` baseline**

Run:

```bash
diff -u /tmp/flag-dependency-exports-baseline.txt /tmp/flag-dependency-exports-feature.txt || true
```

Expected: inspect the diff and keep iterating until the feature branch is overall better than the baseline.

- [ ] **Step 6: If the benchmark is not yet better, make one focused performance cleanup commit**

Use this pattern only if Step 5 still shows a regression:

```rust
let wave_modules: Vec<_> = wave
  .iter()
  .flat_map(|scc_id| artifact.topology().scc_modules(*scc_id).iter().copied())
  .collect();
```

Then rerun the exact commands from Steps 2 through 5.

- [ ] **Step 7: Commit the validated implementation**

Run:

```bash
git add crates/rspack_core crates/rspack_plugin_javascript tests/rspack-test
git commit -m "refactor(javascript): defer dependency export reexports by topology"
```

---

## Suggested Task Dispatch

- Worker 1: Task 2 and Task 3
- Worker 2: Task 4 and Task 5
- Worker 3: Task 6 and Task 7 after Worker 1 and Worker 2 land their interfaces
- Main agent: Task 1 and Task 8, plus review and integration between worker commits

## Self-Review

### Spec coverage

- Two-phase collection and propagation: covered by Tasks 4, 5, and 6.
- Reexport collection-only first pass: covered by Tasks 4 and 5.
- SCC compression and leaf-to-root propagation: covered by Tasks 3 and 6.
- Same-wave parallelism: covered by Tasks 3 and 6.
- Recoverable `FINISH_MODULES` artifact: covered by Task 2.
- Baseline and post-change CodSpeed comparison: covered by Tasks 1 and 8.
- Build, format, clippy, `test:rs`, and `test:unit`: covered by Task 8.

### Placeholder scan

- No unresolved placeholder markers remain.
- Every code-changing task includes concrete file paths, code snippets, and commands.

### Type consistency

- `DependencyExportsAnalysisArtifact`, `ModuleDependencyExportsAnalysis`, `ExportsProcessing`, `DeferredReexportSpec`, `DeferredReexportItem`, and `normalize_exports_spec` are introduced before later tasks rely on them.
