#![allow(clippy::unwrap_used)]
#![allow(clippy::disallowed_methods)]

//! Reproduction for module graph storage growth across watch-mode rebuilds.
//!
//! The example generates a synthetic ESM project with barrel re-exports and
//! `"sideEffects": false`, builds it once, then toggles a fixed set of source
//! files between two versions and rebuilds `ITERATIONS` times, like switching
//! between two git commits under a dev server. After every build it prints the
//! length, live entry count, and byte size of each dense module graph table.
//!
//! The number of modules and dependencies is constant across rebuilds, so the
//! storage size should be bounded too.
//!
//! ```bash
//! cargo run --profile release-debug -p rspack_benchmark --example module_graph_rebuild_memory
//! ```
//!
//! Environment variables:
//! - `ITERATIONS` (default 40): number of rebuilds.
//! - `MODULES` (default 3000): number of generated leaf modules.
//! - `CHANGED` (default 40): number of leaf modules toggled per rebuild.
//! - `ROTATE=1`: edit a different set of files on every rebuild.
//! - `VERBOSE=1`: print every table on every rebuild.
//! - `LOOKUP_LOOP` (default 0): after the initial build, time this many
//!   iterations of the `rust@module_graph_api` lookup mix.

use std::{
  fmt::Write as _,
  fs,
  path::{Path, PathBuf},
  sync::Arc,
  time::Instant,
};

use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  CacheOptions, Compiler, DependencyId, Mode, ModuleGraph, ModuleIdentifier,
  rollback::DenseStorageStats,
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem};
use rspack_tasks::{CompilerContext, within_compiler_context_sync};
use rustc_hash::FxHashSet;

const MODULES_PER_BARREL: usize = 50;
const IMPORTS_PER_MODULE: usize = 6;

fn env_usize(name: &str, default: usize) -> usize {
  std::env::var(name)
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(default)
}

fn leaf_path(dir: &Path, index: usize) -> PathBuf {
  dir.join("src").join("lib").join(format!("m{index}.js"))
}

fn leaf_source(index: usize, modules: usize, variant: bool) -> String {
  let barrels = modules.div_ceil(MODULES_PER_BARREL);
  let mut source = String::new();
  let mut used = Vec::new();
  for offset in 1..=IMPORTS_PER_MODULE {
    let target = (index + offset * 7) % modules;
    if target == index {
      continue;
    }
    let barrel = (target / MODULES_PER_BARREL) % barrels;
    writeln!(
      source,
      "import {{ f{target} as i{offset} }} from '../barrel{barrel}.js';"
    )
    .unwrap();
    used.push(format!("i{offset}"));
  }
  if variant {
    // The second "commit" adds a dependency and changes the body.
    let target = (index + 1) % modules;
    let barrel = (target / MODULES_PER_BARREL) % barrels;
    writeln!(
      source,
      "import {{ g{target} as extra }} from '../barrel{barrel}.js';"
    )
    .unwrap();
    used.push("extra".to_string());
  }
  writeln!(
    source,
    "export function f{index}(n) {{ return n > 0 ? [{}].length + n : {index}; }}",
    used
      .iter()
      .map(|name| format!("typeof {name}"))
      .collect::<Vec<_>>()
      .join(", ")
  )
  .unwrap();
  writeln!(
    source,
    "export const g{index} = {{ id: {index}, variant: {variant} }};"
  )
  .unwrap();
  writeln!(source, "export default f{index};").unwrap();
  source
}

fn generate_project(dir: &Path, modules: usize) {
  let _ = fs::remove_dir_all(dir);
  fs::create_dir_all(dir.join("src").join("lib")).unwrap();
  fs::write(
    dir.join("package.json"),
    r#"{ "name": "module-graph-rebuild-memory", "sideEffects": false }"#,
  )
  .unwrap();

  for index in 0..modules {
    fs::write(leaf_path(dir, index), leaf_source(index, modules, false)).unwrap();
  }

  let barrels = modules.div_ceil(MODULES_PER_BARREL);
  let mut entry = String::new();
  for barrel in 0..barrels {
    let mut source = String::new();
    let start = barrel * MODULES_PER_BARREL;
    let end = (start + MODULES_PER_BARREL).min(modules);
    for index in start..end {
      writeln!(
        source,
        "export {{ f{index}, g{index} }} from './lib/m{index}.js';"
      )
      .unwrap();
    }
    fs::write(dir.join("src").join(format!("barrel{barrel}.js")), source).unwrap();
    writeln!(entry, "import * as b{barrel} from './barrel{barrel}.js';").unwrap();
  }
  writeln!(
    entry,
    "console.log([{}].map(b => Object.keys(b).length));",
    (0..barrels)
      .map(|barrel| format!("b{barrel}"))
      .collect::<Vec<_>>()
      .join(", ")
  )
  .unwrap();
  fs::write(dir.join("src").join("index.js"), entry).unwrap();
}

fn compiler_builder(dir: &Path) -> CompilerBuilder {
  let mut builder = Compiler::builder();
  builder
    .context(dir.to_string_lossy().to_string())
    .mode(Mode::Development)
    .entry("main", "./src/index.js")
    .cache(CacheOptions::Memory { max_generations: 1 })
    .input_filesystem(Arc::new(NativeFileSystem::new(false)))
    .output_filesystem(Arc::new(MemoryFileSystem::default()));
  builder
}

fn mib(bytes: usize) -> f64 {
  bytes as f64 / 1024.0 / 1024.0
}

fn report(compiler: &Compiler, round: usize, elapsed_ms: f64, verbose: bool) {
  let compilation = &compiler.compilation;
  let module_graph = compilation.get_module_graph();
  let stats = module_graph.dense_storage_stats();
  let tables = stats.tables();
  let total = tables
    .iter()
    .fold(DenseStorageStats::default(), |acc, (_, stats)| acc + *stats);
  let errors = compilation.get_errors().count();
  assert_eq!(errors, 0, "compilation should not have errors");
  println!(
    "{round:>5} {elapsed_ms:>9.1} {:>8} {:>9} {:>12} {:>10} {:>10} {:>10} {:>10} {:>10.2}",
    module_graph.modules_len(),
    stats.dependencies.live,
    compilation.exports_info_artifact.len(),
    stats.dependencies.id_space,
    stats.connections.id_space,
    total.slots,
    total.live,
    mib(total.bytes),
  );
  if verbose {
    for (name, stats) in tables {
      println!(
        "      {name:<42} id_space={:>9} live={:>8} paged={:>8} slots={:>9} bytes={:>10}",
        stats.id_space, stats.live, stats.paged, stats.slots, stats.bytes
      );
    }
  }
}

/// The lookup mix of the `rust@module_graph_api` benchmark, kept out of line so
/// profilers such as callgrind can measure it on its own.
#[inline(never)]
fn module_graph_lookup_loop(
  module_graph: &ModuleGraph,
  dependency_ids: &[DependencyId],
  module_ids: &[ModuleIdentifier],
) -> usize {
  let mut found = 0;
  for dependency_id in dependency_ids {
    found += usize::from(
      module_graph
        .connection_by_dependency_id(dependency_id)
        .is_some(),
    );
    found += usize::from(
      module_graph
        .get_module_by_dependency_id(dependency_id)
        .is_some(),
    );
    found += usize::from(module_graph.get_resolved_module(dependency_id).is_some());
  }
  for module_id in module_ids {
    found += module_graph.get_outgoing_connections(module_id).count();
    found += module_graph
      .get_ordered_outgoing_connections(module_id)
      .count();
    found += module_graph.get_incoming_connections(module_id).count();
  }
  found
}

fn run_lookup_loop(compiler: &Compiler, iterations: usize) {
  let module_graph = compiler.compilation.get_module_graph();
  let dependency_ids = module_graph
    .dependencies()
    .map(|(dependency_id, _)| dependency_id)
    .collect::<Vec<_>>();
  let module_ids = module_graph.modules_keys().copied().collect::<Vec<_>>();
  let start = Instant::now();
  let mut found = 0;
  for _ in 0..iterations {
    found += std::hint::black_box(module_graph_lookup_loop(
      module_graph,
      &dependency_ids,
      &module_ids,
    ));
  }
  println!(
    "lookup loop: {iterations} iterations, {:.3} ms/iteration, {found} hits",
    start.elapsed().as_secs_f64() * 1000.0 / iterations.max(1) as f64
  );
}

fn main() {
  let iterations = env_usize("ITERATIONS", 40);
  let modules = env_usize("MODULES", 3000);
  let changed = env_usize("CHANGED", 40).min(modules);
  let verbose = std::env::var("VERBOSE").is_ok_and(|value| value == "1");
  let rotate = std::env::var("ROTATE").is_ok_and(|value| value == "1");

  let dir = std::env::temp_dir().join(format!(
    "rspack-module-graph-rebuild-memory-{}",
    std::process::id()
  ));
  generate_project(&dir, modules);

  let rt = rspack_benchmark::build_tokio_rt();
  let compiler_context = Arc::new(CompilerContext::new());
  let mut compiler =
    within_compiler_context_sync(compiler_context, || compiler_builder(&dir).build().unwrap());

  println!(
    "{:>5} {:>9} {:>8} {:>9} {:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
    "round", "ms", "modules", "deps", "exports_info", "dep_ids", "conn_ids", "slots", "live", "MiB"
  );

  let start = Instant::now();
  rt.block_on(compiler.build()).unwrap();
  let lookup_iterations = env_usize("LOOKUP_LOOP", 0);
  if lookup_iterations > 0 {
    run_lookup_loop(&compiler, lookup_iterations);
  }
  report(
    &compiler,
    0,
    start.elapsed().as_secs_f64() * 1000.0,
    verbose,
  );

  // Spread the toggled files over the whole project. With `ROTATE=1`, every
  // rebuild edits a different set of files instead of the same one.
  let step = (modules / changed.max(1)).max(1);
  let mut variants = vec![false; modules];

  for round in 1..=iterations {
    let shift = if rotate { round } else { 0 };
    let mut changed_files = FxHashSet::default();
    for index in (0..changed).map(|index| (index * step + shift) % modules) {
      variants[index] = !variants[index];
      let path = leaf_path(&dir, index);
      fs::write(&path, leaf_source(index, modules, variants[index])).unwrap();
      changed_files.insert(path.to_string_lossy().to_string());
    }
    let start = Instant::now();
    rt.block_on(compiler.rebuild(changed_files, FxHashSet::default()))
      .unwrap();
    report(
      &compiler,
      round,
      start.elapsed().as_secs_f64() * 1000.0,
      verbose,
    );
  }

  if lookup_iterations > 0 && iterations > 0 {
    // Measure again once dependency ids have churned through the rebuilds.
    run_lookup_loop(&compiler, lookup_iterations);
  }

  rt.block_on(compiler.close()).unwrap();
  let _ = fs::remove_dir_all(&dir);
}
