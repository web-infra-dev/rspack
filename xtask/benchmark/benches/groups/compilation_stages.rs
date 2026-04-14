#![allow(clippy::unwrap_used)]

use std::{
  cell::{Cell, RefCell},
  collections::HashMap,
  hash::Hash,
  sync::Arc,
};

use criterion::{BatchSize, black_box, criterion_group};
use rspack::builder::Builder as _;
use rspack_benchmark::Criterion;
use rspack_collections::IdentifierSet;
use rspack_core::{
  AsyncModulesArtifact, CacheOptions, ChunkByUkey, ChunkContentHash, ChunkGraph,
  ChunkNamedIdArtifact, ChunkUkey, CodeGenerationJob, Compilation, CompilationAsset,
  CompilationAssets, Compiler, DEFAULT_DELIMITER, EntryRuntime, MangleExportsOption, Mode,
  ModuleCodeGenerationContext, ModuleIdsArtifact, Optimization, OutputOptions, RuntimeGlobals,
  RuntimeModule, RuntimeSpecMap, SideEffectsOptimizeArtifact, SourceType, UsedExportsOption,
  build_chunk_graph,
  build_module_graph::{build_module_graph_pass, finish_build_module_graph},
  incremental::IncrementalOptions,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_fs::{MemoryFileSystem, WritableFileSystem};
use rspack_hash::RspackHash;
use rspack_plugin_split_chunks::{
  CacheGroup, CacheGroupTest, ChunkNameGetter, FallbackCacheGroup, PluginOptions, SplitChunkSizes,
  SplitChunksPlugin, create_all_chunk_filter, create_default_module_layer_filter,
  create_default_module_type_filter,
};
use rspack_tasks::within_compiler_context_for_testing_sync;
use rustc_hash::FxHashMap;
use tokio::runtime::Builder;

use crate::groups::build_chunk_graph::prepare_large_code_splitting_case;

const GENERAL_STAGE_NUM_MODULES: usize = 3000;
const CONCAT_GROUPS: usize = 160;
const CONCAT_MODULES_PER_GROUP: usize = 12;
const SPLIT_CHUNKS_ENTRY_COUNT: usize = 48;
const SPLIT_CHUNKS_SHARED_MODULES: usize = 192;
const SPLIT_CHUNKS_WINDOW: usize = 20;
const SPLIT_CHUNKS_COMMON_MODULES: usize = 16;
const MODULE_ASSET_SEED_COUNT: usize = 256;

pub fn compilation_stages_benchmark(c: &mut Criterion) {
  within_compiler_context_for_testing_sync(|| {
    compilation_stages_benchmark_inner(c);
  })
}

fn compilation_stages_benchmark_inner(c: &mut Criterion) {
  let rt = Builder::new_multi_thread()
    .build()
    .expect("should not fail to build tokio runtime");
  let _guard = rt.enter();

  flag_dependency_exports_benchmark(c, &rt);
  flag_dependency_usage_benchmark(c, &rt);
  create_module_ids_benchmark(c, &rt);
  split_chunks_benchmark(c, &rt);
  create_chunk_ids_benchmark(c, &rt);
  mangle_exports_benchmark(c, &rt);
  create_module_hashes_benchmark(c, &rt);
  runtime_requirements_benchmark(c, &rt);
  create_chunk_hashes_benchmark(c, &rt);
  create_full_hash_benchmark(c, &rt);
  create_module_assets_benchmark(c, &rt);
  create_chunk_assets_benchmark(c, &rt);
  real_content_hash_benchmark(c, &rt);
  create_concatenate_module_benchmark(c, &rt);
  concatenate_module_code_generation_benchmark(c, &rt);
}

fn flag_dependency_exports_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_build_module_graph_phase(&mut compiler)
      .await
      .unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "flag_dependency_exports setup");
  let compiler = RefCell::new(compiler);
  let should_reset = Cell::new(false);
  c.bench_function("rust@flag_dependency_exports", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        if should_reset.get() {
          compiler.compilation.exports_info_artifact.reset();
        } else {
          should_reset.set(true);
        }
        compiler.compilation.exports_info_artifact.checkpoint();
        compiler.compilation.async_modules_artifact = AsyncModulesArtifact::default().into();
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_finish_modules_hook(&mut compiler.compilation)
            .await
            .unwrap();
        });
      },
      BatchSize::PerIteration,
    );
  });
}

fn flag_dependency_usage_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_build_module_graph_phase(&mut compiler)
      .await
      .unwrap();
    run_finish_modules_hook(&mut compiler.compilation)
      .await
      .unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "flag_dependency_usage setup");
  let compiler = RefCell::new(compiler);
  let should_reset = Cell::new(false);
  c.bench_function("rust@flag_dependency_usage", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        if should_reset.get() {
          compiler
            .compilation
            .build_module_graph_artifact
            .get_module_graph_mut()
            .reset();
          compiler.compilation.exports_info_artifact.reset();
        } else {
          should_reset.set(true);
        }
        compiler
          .compilation
          .build_module_graph_artifact
          .get_module_graph_mut()
          .checkpoint();
        compiler.compilation.exports_info_artifact.checkpoint();
        compiler.compilation.side_effects_optimize_artifact =
          SideEffectsOptimizeArtifact::default().into();
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_optimize_dependencies_hook(&mut compiler.compilation)
            .await
            .unwrap();
        });
      },
      BatchSize::PerIteration,
    );
  });
}

fn create_module_ids_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_module_ids(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_module_ids setup");

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@create_module_ids", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        compiler.compilation.module_ids_artifact.clear();
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_module_ids_hook(&mut compiler.compilation)
            .await
            .unwrap();
        });
        black_box(compiler.compilation.module_ids_artifact.len());
      },
      BatchSize::PerIteration,
    );
  });
}

fn split_chunks_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let mut compiler = create_split_chunks_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_split_chunks_case(SPLIT_CHUNKS_ENTRY_COUNT, SPLIT_CHUNKS_SHARED_MODULES, &fs)
      .await;
    prepare_for_split_chunks(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "split_chunks setup");

  let initial_chunk_graph = compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .clone();
  let initial_chunk_by_ukey = compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .clone();
  let initial_chunk_group_by_ukey = compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_group_by_ukey
    .clone();
  let initial_entrypoints = compiler
    .compilation
    .build_chunk_graph_artifact
    .entrypoints
    .clone();
  let initial_async_entrypoints = compiler
    .compilation
    .build_chunk_graph_artifact
    .async_entrypoints
    .clone();
  let initial_named_chunk_groups = compiler
    .compilation
    .build_chunk_graph_artifact
    .named_chunk_groups
    .clone();
  let initial_named_chunks = compiler
    .compilation
    .build_chunk_graph_artifact
    .named_chunks
    .clone();
  let chunk_count_before = initial_chunk_by_ukey.len();

  let restore_initial_chunk_state = |compilation: &mut Compilation| {
    compilation.build_chunk_graph_artifact.chunk_graph = initial_chunk_graph.clone();
    compilation.build_chunk_graph_artifact.chunk_by_ukey = initial_chunk_by_ukey.clone();
    compilation.build_chunk_graph_artifact.chunk_group_by_ukey =
      initial_chunk_group_by_ukey.clone();
    compilation.build_chunk_graph_artifact.entrypoints = initial_entrypoints.clone();
    compilation.build_chunk_graph_artifact.async_entrypoints = initial_async_entrypoints.clone();
    compilation.build_chunk_graph_artifact.named_chunk_groups = initial_named_chunk_groups.clone();
    compilation.build_chunk_graph_artifact.named_chunks = initial_named_chunks.clone();
  };

  rt.block_on(async {
    run_optimize_chunks_hook(&mut compiler.compilation)
      .await
      .unwrap();
  });

  let chunk_count_after = compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .len();
  assert!(
    chunk_count_after > chunk_count_before,
    "split_chunks setup should create additional shared chunks"
  );

  restore_initial_chunk_state(&mut compiler.compilation);

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@split_chunks", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        restore_initial_chunk_state(&mut compiler.compilation);
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_optimize_chunks_hook(&mut compiler.compilation)
            .await
            .unwrap();
        });
        black_box(
          compiler
            .compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .len(),
        );
      },
      BatchSize::PerIteration,
    );
  });
}

fn create_chunk_ids_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_chunk_ids(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_chunk_ids setup");
  let initial_chunk_by_ukey = compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .clone();

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@create_chunk_ids", |b| {
    b.iter_batched(
      || {
        (
          initial_chunk_by_ukey.clone(),
          ChunkNamedIdArtifact::default(),
        )
      },
      |(mut chunk_by_ukey, mut named_chunk_ids_artifact)| {
        let compiler = compiler.borrow();
        rt.block_on(async {
          run_chunk_ids_hook(
            &compiler.compilation,
            &mut chunk_by_ukey,
            &mut named_chunk_ids_artifact,
          )
          .await
          .unwrap();
        });
        black_box(named_chunk_ids_artifact.chunk_ids.len());
      },
      BatchSize::PerIteration,
    );
  });
}

fn mangle_exports_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_mangle_exports_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_optimize_code_generation(&mut compiler)
      .await
      .unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "mangle_exports setup");
  compiler
    .compilation
    .build_module_graph_artifact
    .get_module_graph_mut()
    .checkpoint();
  compiler.compilation.exports_info_artifact.checkpoint();

  rt.block_on(async {
    run_optimize_code_generation_hook(&mut compiler.compilation)
      .await
      .unwrap();
  });
  let assigned_export_used_names = count_assigned_export_used_names(&compiler.compilation);
  assert!(
    assigned_export_used_names > 0,
    "mangle_exports setup should assign export used names"
  );
  compiler
    .compilation
    .build_module_graph_artifact
    .get_module_graph_mut()
    .reset();
  compiler.compilation.exports_info_artifact.reset();

  let compiler = RefCell::new(compiler);
  let should_reset = Cell::new(false);
  c.bench_function("rust@mangle_exports", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        if should_reset.get() {
          compiler
            .compilation
            .build_module_graph_artifact
            .get_module_graph_mut()
            .reset();
          compiler.compilation.exports_info_artifact.reset();
        } else {
          should_reset.set(true);
        }
        compiler
          .compilation
          .build_module_graph_artifact
          .get_module_graph_mut()
          .checkpoint();
        compiler.compilation.exports_info_artifact.checkpoint();
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_optimize_code_generation_hook(&mut compiler.compilation)
            .await
            .unwrap();
        });
        black_box(count_assigned_export_used_names(&compiler.compilation));
      },
      BatchSize::PerIteration,
    );
  });
}

fn create_module_hashes_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_module_hashes(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_module_hashes setup");

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@create_module_hashes", |b| {
    b.iter_batched_ref(
      || {
        let _compiler = compiler.borrow();
      },
      |_| {
        let compiler = compiler.borrow();
        let computed =
          rt.block_on(async { compute_module_hashes(&compiler.compilation).await.unwrap() });
        black_box(computed);
      },
      BatchSize::PerIteration,
    );
  });
}

fn create_chunk_hashes_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    compiler.build().await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_chunk_hashes setup");
  let compiler = RefCell::new(compiler);

  c.bench_function("rust@create_chunk_hashes", |b| {
    b.iter(|| {
      let compiler = compiler.borrow();
      let computed =
        rt.block_on(async { compute_chunk_hashes(&compiler.compilation).await.unwrap() });
      black_box(computed);
    });
  });
}

fn runtime_requirements_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_runtime_requirements(&mut compiler)
      .await
      .unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "runtime_requirements setup");
  assert!(
    !compiler.compilation.code_generation_results.is_empty(),
    "runtime_requirements setup should prepare code generation results"
  );
  assert!(
    compiler.compilation.runtime_modules.is_empty(),
    "runtime_requirements setup should not have runtime modules before the pass runs"
  );

  c.bench_function("rust@runtime_requirements", |b| {
    b.iter_batched(
      || {
        let fs = Arc::new(MemoryFileSystem::default());
        let random_table = random_table.clone();
        let mut compiler = create_general_stage_compiler(fs.clone());
        rt.block_on(async {
          fs.create_dir_all("/src".into())
            .await
            .expect("should not fail to create dir");
          prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
          prepare_for_runtime_requirements(&mut compiler)
            .await
            .unwrap();
        });
        compiler
      },
      |mut compiler| {
        rt.block_on(async {
          run_runtime_requirements_pass(&mut compiler).await.unwrap();
        });
        black_box((
          compiler.compilation.runtime_modules.len(),
          compiler.compilation.cgc_runtime_requirements_artifact.len(),
        ));
      },
      BatchSize::PerIteration,
    );
  });
}

fn create_full_hash_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_full_hash(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_full_hash setup");
  assert!(
    !compiler.compilation.chunk_hashes_artifact.is_empty(),
    "create_full_hash setup should prepare chunk hashes"
  );
  let compiler = RefCell::new(compiler);

  c.bench_function("rust@create_full_hash", |b| {
    b.iter(|| {
      let mut compiler = compiler.borrow_mut();
      let full_hash = aggregate_full_hash(&mut compiler.compilation);
      black_box(full_hash);
    });
  });
}

fn create_chunk_assets_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_chunk_assets(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_chunk_assets setup");
  let initial_state = snapshot_chunk_asset_state(&compiler.compilation);
  let initial_totals = chunk_asset_totals(&compiler.compilation);

  rt.block_on(async {
    run_create_chunk_assets_pass(&mut compiler).await.unwrap();
  });
  let rendered_totals = chunk_asset_totals(&compiler.compilation);
  assert!(
    rendered_totals.0 > initial_totals.0,
    "create_chunk_assets setup should render chunk files"
  );
  assert!(
    rendered_totals.1 > initial_totals.1,
    "create_chunk_assets setup should mark chunks as rendered"
  );
  restore_chunk_asset_state(&mut compiler.compilation, &initial_state);

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@create_chunk_assets", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        restore_chunk_asset_state(&mut compiler.compilation, &initial_state);
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_create_chunk_assets_pass(&mut compiler).await.unwrap();
        });
        black_box(chunk_asset_totals(&compiler.compilation));
      },
      BatchSize::PerIteration,
    );
  });
}

fn create_module_assets_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_general_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_module_assets(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_module_assets setup");
  let seeded_module_assets = count_module_assets(&compiler.compilation);
  assert!(
    seeded_module_assets > 0,
    "create_module_assets setup should seed module build_info assets"
  );
  let initial_state = snapshot_chunk_asset_state(&compiler.compilation);
  let initial_asset_count = compiler.compilation.assets().len();
  let initial_totals = chunk_asset_totals(&compiler.compilation);

  rt.block_on(async {
    run_create_module_assets_pass(&mut compiler.compilation)
      .await
      .unwrap();
  });
  let emitted_asset_count = compiler.compilation.assets().len();
  let emitted_totals = chunk_asset_totals(&compiler.compilation);
  assert!(
    emitted_asset_count > initial_asset_count,
    "create_module_assets setup should emit seeded module assets"
  );
  assert!(
    emitted_totals.2 > initial_totals.2,
    "create_module_assets setup should register seeded module assets as chunk auxiliary files"
  );
  restore_chunk_asset_state(&mut compiler.compilation, &initial_state);

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@create_module_assets", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        restore_chunk_asset_state(&mut compiler.compilation, &initial_state);
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_create_module_assets_pass(&mut compiler.compilation)
            .await
            .unwrap();
        });
        black_box((
          compiler.compilation.assets().len(),
          chunk_asset_totals(&compiler.compilation),
        ));
      },
      BatchSize::PerIteration,
    );
  });
}

fn real_content_hash_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let random_table = load_random_table();
  let mut compiler = create_real_content_hash_stage_compiler(fs.clone());

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(GENERAL_STAGE_NUM_MODULES, &random_table, &fs).await;
    prepare_for_real_content_hash(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "real_content_hash setup");
  let initial_state = snapshot_chunk_asset_state(&compiler.compilation);
  let initial_asset_names = sorted_asset_names(&compiler.compilation);
  assert!(
    count_assets_with_content_hash(&compiler.compilation) > 0,
    "real_content_hash setup should produce content-hashed assets"
  );

  rt.block_on(async {
    run_process_assets_pass(&mut compiler.compilation)
      .await
      .unwrap();
  });
  let renamed_asset_names = sorted_asset_names(&compiler.compilation);
  assert_ne!(
    renamed_asset_names, initial_asset_names,
    "real_content_hash setup should rename content-hashed assets during process_assets"
  );
  restore_chunk_asset_state(&mut compiler.compilation, &initial_state);

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@real_content_hash", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        restore_chunk_asset_state(&mut compiler.compilation, &initial_state);
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_process_assets_pass(&mut compiler.compilation)
            .await
            .unwrap();
        });
        black_box(asset_name_fingerprint(&compiler.compilation));
      },
      BatchSize::PerIteration,
    );
  });
}

fn create_concatenate_module_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let mut compiler = create_concatenate_stage_compiler(fs.clone());

  rt.block_on(async {
    prepare_large_concatenation_case(CONCAT_GROUPS, CONCAT_MODULES_PER_GROUP, &fs).await;
    prepare_for_concatenate_module(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_concatenate_module setup");
  compiler
    .compilation
    .build_module_graph_artifact
    .get_module_graph_mut()
    .checkpoint();
  let initial_chunk_graph = compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .clone();
  let initial_chunk_by_ukey = compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .clone();

  rt.block_on(async {
    run_optimize_chunk_modules_hook(&mut compiler.compilation)
      .await
      .unwrap();
  });
  let concatenated_count = count_concatenated_modules(&compiler.compilation);
  assert!(
    concatenated_count > 0,
    "create_concatenate_module setup should produce concatenated modules"
  );
  compiler
    .compilation
    .build_module_graph_artifact
    .get_module_graph_mut()
    .reset();
  compiler.compilation.build_chunk_graph_artifact.chunk_graph = initial_chunk_graph.clone();
  compiler
    .compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey = initial_chunk_by_ukey.clone();

  let compiler = RefCell::new(compiler);
  let should_reset = Cell::new(false);
  c.bench_function("rust@create_concatenate_module", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        if should_reset.get() {
          compiler
            .compilation
            .build_module_graph_artifact
            .get_module_graph_mut()
            .reset();
        } else {
          should_reset.set(true);
        }
        compiler
          .compilation
          .build_module_graph_artifact
          .get_module_graph_mut()
          .checkpoint();
        compiler.compilation.build_chunk_graph_artifact.chunk_graph = initial_chunk_graph.clone();
        compiler
          .compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey = initial_chunk_by_ukey.clone();
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_optimize_chunk_modules_hook(&mut compiler.compilation)
            .await
            .unwrap();
        });
        black_box(count_concatenated_modules(&compiler.compilation));
      },
      BatchSize::PerIteration,
    );
  });
}

fn concatenate_module_code_generation_benchmark(c: &mut Criterion, rt: &tokio::runtime::Runtime) {
  let fs = Arc::new(MemoryFileSystem::default());
  let mut compiler = create_concatenate_stage_compiler(fs.clone());

  rt.block_on(async {
    prepare_large_concatenation_case(CONCAT_GROUPS, CONCAT_MODULES_PER_GROUP, &fs).await;
    compiler.build().await.unwrap();
  });

  assert_no_compilation_errors(
    &compiler.compilation,
    "concatenate_module_code_generation setup",
  );
  let concatenated_modules = compiler
    .compilation
    .get_module_graph()
    .modules()
    .filter_map(|(module_identifier, module)| {
      module.as_concatenated_module().map(|_| *module_identifier)
    })
    .collect::<Vec<_>>();
  assert!(
    !concatenated_modules.is_empty(),
    "concatenate_module_code_generation setup should produce concatenated modules"
  );

  let compiler = RefCell::new(compiler);
  c.bench_function("rust@concatenate_module_code_generation", |b| {
    b.iter(|| {
      let compiler = compiler.borrow();
      let generated = rt.block_on(async {
        compute_concatenated_module_codegen(&compiler.compilation, &concatenated_modules)
          .await
          .unwrap()
      });
      black_box(generated);
    });
  });
}

fn load_random_table() -> Vec<Vec<usize>> {
  serde_json::from_str::<Vec<Vec<usize>>>(include_str!("../build_chunk_graph/random_table.json"))
    .expect("should not fail to parse random table json")
}

fn create_general_stage_compiler(fs: Arc<MemoryFileSystem>) -> Compiler {
  Compiler::builder()
    .context("/")
    .mode(Mode::Development)
    .cache(CacheOptions::Disabled)
    .entry("main", "/src/dynamic-0.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs)
    .optimization(
      Optimization::builder()
        .provided_exports(true)
        .used_exports(UsedExportsOption::True)
        .module_ids("deterministic".to_string())
        .chunk_ids("deterministic".to_string())
        .concatenate_modules(false),
    )
    .incremental(IncrementalOptions::empty_passes())
    .build()
    .unwrap()
}

fn create_mangle_exports_stage_compiler(fs: Arc<MemoryFileSystem>) -> Compiler {
  Compiler::builder()
    .context("/")
    .mode(Mode::Development)
    .cache(CacheOptions::Disabled)
    .entry("main", "/src/dynamic-0.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs)
    .optimization(
      Optimization::builder()
        .provided_exports(true)
        .used_exports(UsedExportsOption::True)
        .mangle_exports(MangleExportsOption::Deterministic)
        .module_ids("deterministic".to_string())
        .chunk_ids("deterministic".to_string())
        .concatenate_modules(false),
    )
    .incremental(IncrementalOptions::empty_passes())
    .build()
    .unwrap()
}

fn create_real_content_hash_stage_compiler(fs: Arc<MemoryFileSystem>) -> Compiler {
  Compiler::builder()
    .context("/")
    .mode(Mode::Development)
    .cache(CacheOptions::Disabled)
    .entry("main", "/src/dynamic-0.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs)
    .output(
      OutputOptions::builder()
        .filename("[name].[contenthash:8].js".into())
        .chunk_filename("[name].[contenthash:8].js".into())
        .css_filename("[name].[contenthash:8].css".into())
        .css_chunk_filename("[name].[contenthash:8].css".into()),
    )
    .optimization(
      Optimization::builder()
        .provided_exports(true)
        .used_exports(UsedExportsOption::True)
        .real_content_hash(true)
        .module_ids("deterministic".to_string())
        .chunk_ids("deterministic".to_string())
        .concatenate_modules(false),
    )
    .incremental(IncrementalOptions::empty_passes())
    .build()
    .unwrap()
}

fn create_split_chunks_stage_compiler(fs: Arc<MemoryFileSystem>) -> Compiler {
  let mut builder = Compiler::builder();
  builder
    .context("/")
    .mode(Mode::Development)
    .cache(CacheOptions::Disabled)
    .input_filesystem(fs.clone())
    .output_filesystem(fs)
    .optimization(
      Optimization::builder()
        .provided_exports(true)
        .used_exports(UsedExportsOption::True)
        .module_ids("deterministic".to_string())
        .chunk_ids("deterministic".to_string())
        .concatenate_modules(false),
    )
    .incremental(IncrementalOptions::empty_passes())
    .plugin(Box::new(create_split_chunks_plugin()));
  for entry_index in 0..SPLIT_CHUNKS_ENTRY_COUNT {
    builder.entry(
      format!("entry-{entry_index}"),
      format!("/src/entries/entry-{entry_index}.js"),
    );
  }
  builder.build().unwrap()
}

fn create_concatenate_stage_compiler(fs: Arc<MemoryFileSystem>) -> Compiler {
  Compiler::builder()
    .context("/")
    .mode(Mode::Development)
    .cache(CacheOptions::Disabled)
    .entry("main", "/src/index.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs)
    .optimization(
      Optimization::builder()
        .provided_exports(true)
        .used_exports(UsedExportsOption::True)
        .module_ids("deterministic".to_string())
        .chunk_ids("deterministic".to_string())
        .concatenate_modules(true),
    )
    .incremental(IncrementalOptions::empty_passes())
    .build()
    .unwrap()
}

async fn prepare_build_module_graph_phase(compiler: &mut Compiler) -> Result<()> {
  let mut compilation_params = compiler.new_compilation_params();
  compiler
    .plugin_driver
    .compiler_hooks
    .this_compilation
    .call(&mut compiler.compilation, &mut compilation_params)
    .await?;
  compiler
    .plugin_driver
    .compiler_hooks
    .compilation
    .call(&mut compiler.compilation, &mut compilation_params)
    .await?;
  compiler
    .plugin_driver
    .compiler_hooks
    .make
    .call(&mut compiler.compilation)
    .await?;
  build_module_graph_pass(&mut compiler.compilation).await?;
  compiler
    .plugin_driver
    .compiler_hooks
    .finish_make
    .call(&mut compiler.compilation)
    .await?;

  let make_artifact = compiler.compilation.build_module_graph_artifact.steal();
  let exports_info_artifact = compiler.compilation.exports_info_artifact.steal();
  let (make_artifact, exports_info_artifact) =
    finish_build_module_graph(&compiler.compilation, make_artifact, exports_info_artifact).await?;
  compiler.compilation.build_module_graph_artifact = make_artifact.into();
  compiler.compilation.exports_info_artifact = exports_info_artifact.into();

  Ok(())
}

async fn prepare_for_module_ids(compiler: &mut Compiler) -> Result<()> {
  prepare_build_module_graph_phase(compiler).await?;
  run_finish_modules_hook(&mut compiler.compilation).await?;
  run_seal_hook(&mut compiler.compilation).await?;
  run_optimize_dependencies_hook(&mut compiler.compilation).await?;
  build_chunk_graph::build_chunk_graph(&mut compiler.compilation)?;
  run_optimize_modules_hook(&mut compiler.compilation).await?;
  run_optimize_chunks_hook(&mut compiler.compilation).await?;
  run_optimize_tree_hook(&mut compiler.compilation).await?;
  run_optimize_chunk_modules_hook(&mut compiler.compilation).await?;
  Ok(())
}

async fn prepare_for_split_chunks(compiler: &mut Compiler) -> Result<()> {
  prepare_build_module_graph_phase(compiler).await?;
  run_finish_modules_hook(&mut compiler.compilation).await?;
  run_seal_hook(&mut compiler.compilation).await?;
  run_optimize_dependencies_hook(&mut compiler.compilation).await?;
  build_chunk_graph::build_chunk_graph(&mut compiler.compilation)?;
  run_optimize_modules_hook(&mut compiler.compilation).await?;
  Ok(())
}

async fn prepare_for_chunk_ids(compiler: &mut Compiler) -> Result<()> {
  prepare_for_module_ids(compiler).await?;
  run_module_ids_hook(&mut compiler.compilation).await?;
  Ok(())
}

async fn prepare_for_optimize_code_generation(compiler: &mut Compiler) -> Result<()> {
  prepare_for_chunk_ids(compiler).await?;
  run_chunk_ids_on_compilation(&mut compiler.compilation).await?;
  run_assign_runtime_ids(&mut compiler.compilation)?;
  Ok(())
}

async fn prepare_for_module_hashes(compiler: &mut Compiler) -> Result<()> {
  prepare_for_chunk_ids(compiler).await?;
  run_chunk_ids_on_compilation(&mut compiler.compilation).await?;
  Ok(())
}

async fn prepare_for_runtime_requirements(compiler: &mut Compiler) -> Result<()> {
  prepare_for_optimize_code_generation(compiler).await?;
  run_optimize_code_generation_hook(&mut compiler.compilation).await?;
  run_create_module_hashes_pass(compiler).await?;
  run_code_generation_pass(compiler).await?;
  Ok(())
}

async fn prepare_for_full_hash(compiler: &mut Compiler) -> Result<()> {
  prepare_for_runtime_requirements(compiler).await?;
  run_runtime_requirements_pass(compiler).await?;
  run_pre_full_hash_setup_on_compilation(&mut compiler.compilation).await?;
  Ok(())
}

async fn prepare_for_module_assets(compiler: &mut Compiler) -> Result<()> {
  prepare_for_runtime_requirements(compiler).await?;
  run_runtime_requirements_pass(compiler).await?;
  run_create_hash_pass(compiler).await?;
  let seeded_module_assets = seed_module_assets(&mut compiler.compilation);
  assert!(
    seeded_module_assets > 0,
    "create_module_assets setup should seed module build_info assets"
  );
  Ok(())
}

async fn prepare_for_chunk_assets(compiler: &mut Compiler) -> Result<()> {
  prepare_for_module_assets(compiler).await?;
  run_create_module_assets_pass(&mut compiler.compilation).await?;
  Ok(())
}

async fn prepare_for_real_content_hash(compiler: &mut Compiler) -> Result<()> {
  prepare_for_chunk_assets(compiler).await?;
  run_create_chunk_assets_pass(compiler).await?;
  Ok(())
}

async fn prepare_for_concatenate_module(compiler: &mut Compiler) -> Result<()> {
  prepare_build_module_graph_phase(compiler).await?;
  run_finish_modules_hook(&mut compiler.compilation).await?;
  run_seal_hook(&mut compiler.compilation).await?;
  run_optimize_dependencies_hook(&mut compiler.compilation).await?;
  build_chunk_graph::build_chunk_graph(&mut compiler.compilation)?;
  run_optimize_modules_hook(&mut compiler.compilation).await?;
  run_optimize_chunks_hook(&mut compiler.compilation).await?;
  run_optimize_tree_hook(&mut compiler.compilation).await?;
  Ok(())
}

async fn run_finish_modules_hook(compilation: &mut Compilation) -> Result<()> {
  let mut async_modules_artifact = compilation.async_modules_artifact.steal();
  let mut exports_info_artifact = compilation.exports_info_artifact.steal();
  let mut side_effects_state_artifact = std::mem::take(
    &mut compilation
      .build_module_graph_artifact
      .side_effects_state_artifact,
  );
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .finish_modules
    .call(
      compilation,
      &mut async_modules_artifact,
      &mut exports_info_artifact,
      &mut side_effects_state_artifact,
    )
    .await?;
  compilation.async_modules_artifact = async_modules_artifact.into();
  compilation.exports_info_artifact = exports_info_artifact.into();
  compilation
    .build_module_graph_artifact
    .side_effects_state_artifact = side_effects_state_artifact;
  Ok(())
}

async fn run_seal_hook(compilation: &mut Compilation) -> Result<()> {
  let mut diagnostics = vec![];
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .seal
    .call(compilation, &mut diagnostics)
    .await?;
  assert!(
    diagnostics.is_empty(),
    "seal benchmark setup should not produce diagnostics"
  );
  Ok(())
}

async fn run_optimize_dependencies_hook(compilation: &mut Compilation) -> Result<()> {
  let mut diagnostics: Vec<Diagnostic> = vec![];
  let mut side_effects_optimize_artifact = compilation.side_effects_optimize_artifact.steal();
  let mut build_module_graph_artifact = compilation.build_module_graph_artifact.steal();
  let mut exports_info_artifact = compilation.exports_info_artifact.steal();
  while matches!(
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .optimize_dependencies
      .call(
        compilation,
        &mut side_effects_optimize_artifact,
        &mut build_module_graph_artifact,
        &mut exports_info_artifact,
        &mut diagnostics
      )
      .await?,
    Some(true)
  ) {}
  compilation.side_effects_optimize_artifact = side_effects_optimize_artifact.into();
  compilation.build_module_graph_artifact = build_module_graph_artifact.into();
  compilation.exports_info_artifact = exports_info_artifact.into();
  assert!(
    diagnostics.is_empty(),
    "optimize_dependencies benchmark setup should not produce diagnostics"
  );
  Ok(())
}

async fn run_optimize_modules_hook(compilation: &mut Compilation) -> Result<()> {
  let mut diagnostics = vec![];
  while matches!(
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .optimize_modules
      .call(compilation, &mut diagnostics)
      .await?,
    Some(true)
  ) {}
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .after_optimize_modules
    .call(compilation)
    .await?;
  assert!(
    diagnostics.is_empty(),
    "optimize_modules benchmark setup should not produce diagnostics"
  );
  Ok(())
}

async fn run_optimize_chunks_hook(compilation: &mut Compilation) -> Result<()> {
  while matches!(
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .optimize_chunks
      .call(compilation)
      .await?,
    Some(true)
  ) {}
  Ok(())
}

async fn run_optimize_tree_hook(compilation: &mut Compilation) -> Result<()> {
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .optimize_tree
    .call(compilation)
    .await?;
  Ok(())
}

async fn run_optimize_chunk_modules_hook(compilation: &mut Compilation) -> Result<()> {
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .optimize_chunk_modules
    .call(compilation)
    .await?;
  Ok(())
}

fn run_assign_runtime_ids(compilation: &mut Compilation) -> Result<()> {
  fn process_entrypoint(
    entrypoint_ukey: &rspack_core::ChunkGroupUkey,
    chunk_group_by_ukey: &rspack_core::ChunkGroupByUkey,
    chunk_by_ukey: &ChunkByUkey,
    chunk_graph: &mut ChunkGraph,
  ) {
    let entrypoint = chunk_group_by_ukey.expect_get(entrypoint_ukey);
    let runtime = entrypoint
      .kind
      .get_entry_options()
      .and_then(|entry_options| match &entry_options.runtime {
        Some(EntryRuntime::String(runtime)) => Some(runtime.to_owned()),
        _ => None,
      })
      .or_else(|| entrypoint.name().map(|name| name.to_string()));
    if let (Some(runtime), Some(chunk)) = (
      runtime,
      chunk_by_ukey.get(&entrypoint.get_runtime_chunk(chunk_group_by_ukey)),
    ) {
      chunk_graph.set_runtime_id(runtime, chunk.id().map(|id| id.to_string()));
    }
  }

  for (_, entrypoint_ukey) in &compilation.build_chunk_graph_artifact.entrypoints {
    process_entrypoint(
      entrypoint_ukey,
      &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      &compilation.build_chunk_graph_artifact.chunk_by_ukey,
      &mut compilation.build_chunk_graph_artifact.chunk_graph,
    );
  }
  for entrypoint_ukey in &compilation.build_chunk_graph_artifact.async_entrypoints {
    process_entrypoint(
      entrypoint_ukey,
      &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      &compilation.build_chunk_graph_artifact.chunk_by_ukey,
      &mut compilation.build_chunk_graph_artifact.chunk_graph,
    );
  }

  Ok(())
}

async fn run_optimize_code_generation_hook(compilation: &mut Compilation) -> Result<()> {
  let mut build_module_graph_artifact = compilation.build_module_graph_artifact.steal();
  let mut exports_info_artifact = compilation.exports_info_artifact.steal();
  let mut diagnostics = vec![];
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .optimize_code_generation
    .call(
      compilation,
      &mut build_module_graph_artifact,
      &mut exports_info_artifact,
      &mut diagnostics,
    )
    .await?;
  compilation.build_module_graph_artifact = build_module_graph_artifact.into();
  compilation.exports_info_artifact = exports_info_artifact.into();
  assert!(
    diagnostics.is_empty(),
    "optimize_code_generation benchmark setup should not produce diagnostics"
  );
  Ok(())
}

fn get_modules_needing_ids(
  compilation: &Compilation,
  module_ids_artifact: &ModuleIdsArtifact,
) -> IdentifierSet {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  compilation
    .get_module_graph()
    .modules()
    .map(|(_, module)| module)
    .filter(|module| {
      module.need_id()
        && ChunkGraph::get_module_id(module_ids_artifact, module.identifier()).is_none()
        && chunk_graph.get_number_of_module_chunks(module.identifier()) != 0
    })
    .map(|module| module.identifier())
    .collect()
}

async fn run_module_ids_hook(compilation: &mut Compilation) -> Result<()> {
  let mut module_ids_artifact = compilation.module_ids_artifact.steal();
  let modules_needing_ids = get_modules_needing_ids(compilation, &module_ids_artifact);
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .before_module_ids
    .call(compilation, &modules_needing_ids, &mut module_ids_artifact)
    .await?;
  compilation.module_ids_artifact = module_ids_artifact.into();

  let mut diagnostics = vec![];
  let mut module_ids_artifact = compilation.module_ids_artifact.steal();
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .module_ids
    .call(compilation, &mut module_ids_artifact, &mut diagnostics)
    .await?;
  compilation.module_ids_artifact = module_ids_artifact.into();
  assert!(
    diagnostics.is_empty(),
    "module_ids benchmark setup should not produce diagnostics"
  );
  Ok(())
}

async fn run_chunk_ids_hook(
  compilation: &Compilation,
  chunk_by_ukey: &mut ChunkByUkey,
  named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
) -> Result<()> {
  let mut diagnostics = vec![];
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .chunk_ids
    .call(
      compilation,
      chunk_by_ukey,
      named_chunk_ids_artifact,
      &mut diagnostics,
    )
    .await?;
  assert!(
    diagnostics.is_empty(),
    "chunk_ids benchmark setup should not produce diagnostics"
  );
  Ok(())
}

async fn run_chunk_ids_on_compilation(compilation: &mut Compilation) -> Result<()> {
  let mut chunk_by_ukey = std::mem::take(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
  let mut named_chunk_ids_artifact = compilation.named_chunk_ids_artifact.steal();
  run_chunk_ids_hook(
    compilation,
    &mut chunk_by_ukey,
    &mut named_chunk_ids_artifact,
  )
  .await?;
  compilation.build_chunk_graph_artifact.chunk_by_ukey = chunk_by_ukey;
  compilation.named_chunk_ids_artifact = named_chunk_ids_artifact.into();
  Ok(())
}

async fn run_create_module_hashes_on_compilation(compilation: &mut Compilation) -> Result<()> {
  let module_identifiers = compilation
    .get_module_graph()
    .modules_keys()
    .copied()
    .collect::<Vec<_>>();

  for module_identifier in module_identifiers {
    let runtimes = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_runtimes_iter(
        module_identifier,
        &compilation.build_chunk_graph_artifact.chunk_by_ukey,
      )
      .cloned()
      .collect::<Vec<_>>();
    let mut hashes = RuntimeSpecMap::new();
    {
      let module_graph = compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(&module_identifier)
        .expect("should have module");
      for runtime in &runtimes {
        let hash = module.get_runtime_hash(compilation, Some(runtime)).await?;
        hashes.set(runtime.clone(), hash);
      }
    }
    ChunkGraph::set_module_hashes(compilation, module_identifier, hashes);
  }

  Ok(())
}

async fn run_create_module_hashes_pass(compiler: &mut Compiler) -> Result<()> {
  compiler
    .cache
    .before_modules_hashes(&mut compiler.compilation)
    .await;
  run_create_module_hashes_on_compilation(&mut compiler.compilation).await?;
  compiler
    .cache
    .after_modules_hashes(&compiler.compilation)
    .await;
  Ok(())
}

async fn run_code_generation_on_compilation(compilation: &mut Compilation) -> Result<()> {
  *compilation.code_generation_results = Default::default();
  compilation.code_generated_modules.clear();

  let module_identifiers = compilation
    .get_module_graph()
    .modules_keys()
    .copied()
    .collect::<Vec<_>>();

  for module_identifier in module_identifiers {
    let runtimes = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_runtimes_iter(
        module_identifier,
        &compilation.build_chunk_graph_artifact.chunk_by_ukey,
      )
      .cloned()
      .collect::<Vec<_>>();

    for runtime in runtimes {
      let hash = ChunkGraph::get_module_hash(compilation, module_identifier, &runtime)
        .expect("should have module hash before code generation")
        .clone();
      let concatenation_scope = compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .concatenation_scope
        .call(compilation, module_identifier)
        .await?;
      let mut runtime_template = compilation.runtime_template.create_module_code_template();
      let mut code_generation_context = ModuleCodeGenerationContext {
        compilation,
        runtime: Some(&runtime),
        concatenation_scope,
        runtime_template: &mut runtime_template,
      };
      let module_graph = compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(&module_identifier)
        .expect("should have module");
      let mut code_generation_result = module.code_generation(&mut code_generation_context).await?;
      code_generation_result
        .runtime_requirements
        .extend(*runtime_template.runtime_requirements());
      if module.as_concatenated_module().is_some() {
        code_generation_result.set_hash_for_concatenated_module(
          &hash,
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
      } else {
        code_generation_result.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
      }
      compilation.code_generation_results.insert(
        module_identifier,
        code_generation_result,
        [runtime],
      );
    }

    compilation.code_generated_modules.insert(module_identifier);
  }

  let mut diagnostics = vec![];
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .after_code_generation
    .call(compilation, &mut diagnostics)
    .await?;
  compilation.extend_diagnostics(diagnostics);

  Ok(())
}

async fn run_code_generation_pass(compiler: &mut Compiler) -> Result<()> {
  compiler
    .cache
    .before_modules_codegen(&mut compiler.compilation)
    .await;
  run_code_generation_on_compilation(&mut compiler.compilation).await?;
  compiler
    .cache
    .after_modules_codegen(&compiler.compilation)
    .await;
  Ok(())
}

#[derive(Clone)]
struct ChunkAssetStateSnapshot {
  assets: CompilationAssets,
  chunk_by_ukey: ChunkByUkey,
}

fn snapshot_chunk_asset_state(compilation: &Compilation) -> ChunkAssetStateSnapshot {
  ChunkAssetStateSnapshot {
    assets: compilation.assets().clone(),
    chunk_by_ukey: compilation.build_chunk_graph_artifact.chunk_by_ukey.clone(),
  }
}

fn restore_chunk_asset_state(compilation: &mut Compilation, snapshot: &ChunkAssetStateSnapshot) {
  *compilation.assets_mut() = snapshot.assets.clone();
  compilation.build_chunk_graph_artifact.chunk_by_ukey = snapshot.chunk_by_ukey.clone();
}

fn chunk_asset_totals(compilation: &Compilation) -> (usize, usize, usize) {
  let mut emitted_files = 0;
  let mut rendered_chunks = 0;
  let mut auxiliary_files = 0;

  for chunk in compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
  {
    emitted_files += chunk.files().len();
    auxiliary_files += chunk.auxiliary_files().len();
    rendered_chunks += usize::from(chunk.rendered());
  }

  (emitted_files, rendered_chunks, auxiliary_files)
}

fn count_module_assets(compilation: &Compilation) -> usize {
  compilation
    .get_module_graph()
    .modules()
    .map(|(_, module)| module.build_info().assets.len())
    .sum()
}

fn sorted_asset_names(compilation: &Compilation) -> Vec<String> {
  let mut asset_names = compilation.assets().keys().cloned().collect::<Vec<_>>();
  asset_names.sort_unstable();
  asset_names
}

fn asset_name_fingerprint(compilation: &Compilation) -> usize {
  sorted_asset_names(compilation)
    .into_iter()
    .map(|asset_name| asset_name.bytes().map(usize::from).sum::<usize>())
    .sum()
}

fn count_assets_with_content_hash(compilation: &Compilation) -> usize {
  compilation
    .assets()
    .values()
    .filter(|asset| !asset.get_info().content_hash.is_empty())
    .count()
}

async fn run_create_hash_pass(compiler: &mut Compiler) -> Result<()> {
  compiler
    .cache
    .before_chunks_hashes(&mut compiler.compilation)
    .await;
  run_create_hash_on_compilation(&mut compiler.compilation).await?;
  compiler
    .cache
    .after_chunks_hashes(&compiler.compilation)
    .await;
  Ok(())
}

async fn run_pre_full_hash_setup_on_compilation(compilation: &mut Compilation) -> Result<()> {
  compilation.chunk_hashes_artifact.clear();
  compilation.runtime_modules_hash.clear();
  compilation.hash = None;

  populate_runtime_module_hashes(compilation).await?;

  let chunk_ukeys = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<Vec<_>>();
  for chunk_ukey in chunk_ukeys {
    let (chunk_hash, content_hash) = process_chunk_hash(compilation, chunk_ukey).await?;
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey);
    chunk.set_hashes(
      &mut compilation.chunk_hashes_artifact,
      chunk_hash,
      content_hash,
    );
  }

  Ok(())
}

async fn run_create_hash_on_compilation(compilation: &mut Compilation) -> Result<()> {
  run_pre_full_hash_setup_on_compilation(compilation).await?;
  aggregate_full_hash(compilation);
  run_runtime_modules_code_generation_on_compilation(compilation).await?;
  Ok(())
}

async fn populate_runtime_module_hashes(compilation: &mut Compilation) -> Result<()> {
  let runtime_module_identifiers = compilation
    .runtime_modules
    .keys()
    .copied()
    .collect::<Vec<_>>();

  for runtime_module_identifier in runtime_module_identifiers {
    let digest = {
      let runtime_module = compilation
        .runtime_modules
        .get(&runtime_module_identifier)
        .expect("should have runtime module");
      runtime_module.get_runtime_hash(compilation, None).await?
    };
    compilation
      .runtime_modules_hash
      .insert(runtime_module_identifier, digest);
  }

  Ok(())
}

async fn run_runtime_modules_code_generation_on_compilation(
  compilation: &mut Compilation,
) -> Result<()> {
  compilation.runtime_modules_code_generation_source.clear();

  let runtime_module_identifiers = compilation
    .runtime_modules
    .keys()
    .copied()
    .collect::<Vec<_>>();
  for runtime_module_identifier in runtime_module_identifiers.iter().copied() {
    let source = {
      let runtime_module = compilation
        .runtime_modules
        .get(&runtime_module_identifier)
        .expect("should have runtime module");
      let mut runtime_template = compilation.runtime_template.create_module_code_template();
      let mut code_generation_context = ModuleCodeGenerationContext {
        compilation,
        runtime: None,
        concatenation_scope: None,
        runtime_template: &mut runtime_template,
      };
      let code_generation_result = runtime_module
        .code_generation(&mut code_generation_context)
        .await?;
      code_generation_result
        .get(&SourceType::Runtime)
        .expect("runtime module should emit runtime source")
        .clone()
    };
    compilation
      .runtime_modules_code_generation_source
      .insert(runtime_module_identifier, source);
  }

  compilation
    .code_generated_modules
    .extend(runtime_module_identifiers);
  Ok(())
}

async fn run_create_module_assets_pass(compilation: &mut Compilation) -> Result<()> {
  let mut chunk_asset_map = vec![];
  let mut module_assets = vec![];
  let module_graph = compilation.get_module_graph();

  for (identifier, module) in module_graph.modules() {
    let assets = &module.build_info().assets;
    if assets.is_empty() {
      continue;
    }

    for (name, asset) in assets.as_ref() {
      module_assets.push((name.clone(), asset.clone()));
    }

    if compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_number_of_module_chunks(*identifier)
      > 0
    {
      for chunk in compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(*identifier)
        .iter()
      {
        for name in assets.keys() {
          chunk_asset_map.push((*chunk, name.clone()));
        }
      }
    }
  }

  for (name, asset) in module_assets {
    compilation.emit_asset(name, asset);
  }

  for (chunk, asset_name) in chunk_asset_map {
    compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get_mut(&chunk)
      .add_auxiliary_file(asset_name);
  }

  Ok(())
}

fn seed_module_assets(compilation: &mut Compilation) -> usize {
  let module_identifiers = compilation
    .get_module_graph()
    .modules_keys()
    .copied()
    .filter(|module_identifier| {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_number_of_module_chunks(*module_identifier)
        > 0
    })
    .take(MODULE_ASSET_SEED_COUNT)
    .collect::<Vec<_>>();

  for (asset_index, module_identifier) in module_identifiers.iter().copied().enumerate() {
    let module = compilation
      .get_module_graph_mut()
      .module_by_identifier_mut(&module_identifier)
      .expect("seeded module should exist");
    module.build_info_mut().assets.insert(
      format!("module-assets/module-{asset_index}.txt"),
      CompilationAsset::new(
        Some(RawStringSource::from(format!("module asset fixture {asset_index}")).boxed()),
        Default::default(),
      ),
    );
  }

  module_identifiers.len()
}

async fn run_create_chunk_assets_pass(compiler: &mut Compiler) -> Result<()> {
  compiler
    .cache
    .before_chunk_asset(&mut compiler.compilation)
    .await;
  run_create_chunk_assets_on_compilation(&mut compiler.compilation).await?;
  compiler
    .cache
    .after_chunk_asset(&compiler.compilation)
    .await;
  Ok(())
}

async fn run_create_chunk_assets_on_compilation(compilation: &mut Compilation) -> Result<()> {
  let plugin_driver = compilation.plugin_driver.clone();
  let chunk_ukeys = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<Vec<_>>();
  let mut chunk_render_results = Vec::with_capacity(chunk_ukeys.len());

  for chunk_ukey in chunk_ukeys {
    let mut manifests = Vec::new();
    let mut diagnostics = Vec::new();
    plugin_driver
      .compilation_hooks
      .render_manifest
      .call(compilation, &chunk_ukey, &mut manifests, &mut diagnostics)
      .await?;
    chunk_render_results.push((chunk_ukey, manifests, diagnostics));
  }

  for (chunk_ukey, manifests, diagnostics) in chunk_render_results {
    compilation.extend_diagnostics(diagnostics);

    for file_manifest in manifests {
      let filename = file_manifest.filename;
      {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get_mut(&chunk_ukey);
        chunk.set_rendered(true);
        if file_manifest.auxiliary {
          chunk.add_auxiliary_file(filename.clone());
        } else {
          chunk.add_file(filename.clone());
        }
      }

      compilation.emit_asset(
        filename.clone(),
        CompilationAsset::new(Some(file_manifest.source), file_manifest.info),
      );

      plugin_driver
        .compilation_hooks
        .chunk_asset
        .call(compilation, &chunk_ukey, &filename)
        .await?;
    }
  }

  Ok(())
}

async fn run_process_assets_pass(compilation: &mut Compilation) -> Result<()> {
  let plugin_driver = compilation.plugin_driver.clone();
  plugin_driver
    .compilation_hooks
    .process_assets
    .call(compilation)
    .await
}

async fn compute_module_hashes(compilation: &Compilation) -> Result<usize> {
  let module_graph = compilation.get_module_graph();
  let mut total = 0;
  let module_identifiers = module_graph.modules_keys().copied().collect::<Vec<_>>();

  for module_identifier in module_identifiers {
    let module = module_graph
      .module_by_identifier(&module_identifier)
      .expect("should have module");
    for runtime in compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_runtimes_iter(
        module_identifier,
        &compilation.build_chunk_graph_artifact.chunk_by_ukey,
      )
    {
      let hash = module.get_runtime_hash(compilation, Some(runtime)).await?;
      black_box(hash);
      total += 1;
    }
  }

  Ok(total)
}

async fn compute_chunk_hashes(compilation: &Compilation) -> Result<usize> {
  let chunk_ukeys = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<Vec<_>>();
  let mut total = 0;

  for chunk_ukey in chunk_ukeys {
    let result = process_chunk_hash(compilation, chunk_ukey).await?;
    black_box(result);
    total += 1;
  }

  Ok(total)
}

async fn run_runtime_requirements_on_compilation(compilation: &mut Compilation) -> Result<()> {
  let plugin_driver = compilation.plugin_driver.clone();
  let handle = tokio::runtime::Handle::current();
  let modules = compilation
    .get_module_graph()
    .modules_keys()
    .copied()
    .filter(|module| {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_number_of_module_chunks(*module)
        > 0
    })
    .collect::<Vec<_>>();

  let module_results = if modules.is_empty() {
    Vec::new()
  } else {
    let worker_count = std::thread::available_parallelism()
      .map(|parallelism| parallelism.get())
      .unwrap_or(1)
      .min(modules.len());
    let chunk_size = modules.len().div_ceil(worker_count);
    std::thread::scope(|scope| -> Result<Vec<_>> {
      let mut workers = Vec::new();
      for module_chunk in modules.chunks(chunk_size) {
        let module_chunk = module_chunk.to_vec();
        let compilation = &*compilation;
        let plugin_driver = plugin_driver.clone();
        let handle = handle.clone();
        workers.push(
          scope.spawn(move || -> Result<Vec<(rspack_core::ModuleIdentifier, RuntimeSpecMap<RuntimeGlobals>)>> {
            let mut local_results = Vec::with_capacity(module_chunk.len());
            for module in module_chunk {
              let runtimes = compilation
                .build_chunk_graph_artifact
                .chunk_graph
                .get_module_runtimes_iter(
                  module,
                  &compilation.build_chunk_graph_artifact.chunk_by_ukey,
                )
                .cloned()
                .collect::<Vec<_>>();
              let mut map = RuntimeSpecMap::new();

              for runtime in runtimes {
                let runtime_requirements = handle.block_on(async {
                  compilation
                    .process_runtime_requirements_cache_artifact
                    .use_cache(module, &runtime, compilation, || async {
                      let mut all_runtime_requirements = compilation
                        .code_generation_results
                        .get_runtime_requirements(&module, Some(&runtime));

                      plugin_driver
                        .compilation_hooks
                        .additional_module_runtime_requirements
                        .call(compilation, &module, &mut all_runtime_requirements)
                        .await?;

                      let mut runtime_requirements_added = all_runtime_requirements;
                      loop {
                        let current_runtime_requirements = runtime_requirements_added;
                        let mut runtime_requirements_to_add = RuntimeGlobals::default();
                        plugin_driver
                          .compilation_hooks
                          .runtime_requirement_in_module
                          .call(
                            compilation,
                            &module,
                            &all_runtime_requirements,
                            &current_runtime_requirements,
                            &mut runtime_requirements_to_add,
                          )
                          .await?;
                        runtime_requirements_to_add = runtime_requirements_to_add
                          .difference(all_runtime_requirements.intersection(runtime_requirements_to_add));
                        if runtime_requirements_to_add.is_empty() {
                          break;
                        }
                        all_runtime_requirements.insert(runtime_requirements_to_add);
                        runtime_requirements_added = runtime_requirements_to_add;
                      }

                      Ok(all_runtime_requirements)
                    })
                    .await
                })?;
                map.set(runtime, runtime_requirements);
              }
              local_results.push((module, map));
            }
            Ok(local_results)
          }),
        );
      }

      let mut combined_results = Vec::with_capacity(modules.len());
      for worker in workers {
        let mut local_results = worker
          .join()
          .expect("runtime requirements module worker should not panic")?;
        combined_results.append(&mut local_results);
      }
      Ok(combined_results)
    })?
  };

  for (module, map) in module_results {
    ChunkGraph::set_module_runtime_requirements(compilation, module, map);
  }

  let entries = compilation
    .get_chunk_graph_entries()
    .collect::<rustc_hash::FxHashSet<_>>();
  let chunks = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<rustc_hash::FxHashSet<_>>();
  let chunk_keys = chunks
    .iter()
    .chain(entries.iter())
    .copied()
    .collect::<Vec<_>>();

  let chunk_requirements = if chunk_keys.is_empty() {
    FxHashMap::default()
  } else {
    let worker_count = std::thread::available_parallelism()
      .map(|parallelism| parallelism.get())
      .unwrap_or(1)
      .min(chunk_keys.len());
    let chunk_size = chunk_keys.len().div_ceil(worker_count);
    std::thread::scope(|scope| -> FxHashMap<ChunkUkey, RuntimeGlobals> {
      let mut workers = Vec::new();
      for chunk_key_chunk in chunk_keys.chunks(chunk_size) {
        let chunk_key_chunk = chunk_key_chunk.to_vec();
        let compilation = &*compilation;
        workers.push(scope.spawn(move || {
          let mut local_results = Vec::with_capacity(chunk_key_chunk.len());
          for chunk_ukey in chunk_key_chunk {
            let mut set = RuntimeGlobals::default();
            for module_identifier in compilation
              .build_chunk_graph_artifact
              .chunk_graph
              .get_chunk_modules_identifier(&chunk_ukey)
            {
              let chunk = compilation
                .build_chunk_graph_artifact
                .chunk_by_ukey
                .expect_get(&chunk_ukey);
              if let Some(runtime_requirements) = ChunkGraph::get_module_runtime_requirements(
                compilation,
                *module_identifier,
                chunk.runtime(),
              ) {
                set.insert(*runtime_requirements);
              }
            }
            local_results.push((chunk_ukey, set));
          }
          local_results
        }));
      }

      let mut combined_results = FxHashMap::default();
      for worker in workers {
        for (chunk_ukey, runtime_requirements) in worker
          .join()
          .expect("runtime requirements chunk worker should not panic")
        {
          combined_results.insert(chunk_ukey, runtime_requirements);
        }
      }
      combined_results
    })
  };

  for (chunk_ukey, mut all_runtime_requirements) in chunk_requirements {
    let mut additional_runtime_modules = Vec::new();
    plugin_driver
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .call(
        compilation,
        &chunk_ukey,
        &mut all_runtime_requirements,
        &mut additional_runtime_modules,
      )
      .await?;

    for module in additional_runtime_modules {
      let additional_runtime_requirements = module.additional_runtime_requirements(compilation);
      all_runtime_requirements.extend(additional_runtime_requirements);
      compilation.add_runtime_module(&chunk_ukey, module)?;
    }

    let mut runtime_modules_to_add = Vec::<Box<dyn RuntimeModule>>::new();
    let mut runtime_requirements_added = all_runtime_requirements;
    loop {
      let current_runtime_requirements = runtime_requirements_added;
      let mut runtime_requirements_to_add = RuntimeGlobals::default();
      plugin_driver
        .compilation_hooks
        .runtime_requirement_in_chunk
        .call(
          compilation,
          &chunk_ukey,
          &all_runtime_requirements,
          &current_runtime_requirements,
          &mut runtime_requirements_to_add,
          &mut runtime_modules_to_add,
        )
        .await?;
      for runtime_module in &runtime_modules_to_add {
        let additional_runtime_requirements =
          runtime_module.additional_runtime_requirements(compilation);
        runtime_requirements_to_add.extend(additional_runtime_requirements);
      }
      runtime_requirements_to_add = runtime_requirements_to_add
        .difference(all_runtime_requirements.intersection(runtime_requirements_to_add));
      if runtime_requirements_to_add.is_empty() {
        break;
      }
      all_runtime_requirements.insert(runtime_requirements_to_add);
      runtime_requirements_added = runtime_requirements_to_add;
    }

    for module in runtime_modules_to_add {
      compilation.add_runtime_module(&chunk_ukey, module)?;
    }

    ChunkGraph::set_chunk_runtime_requirements(compilation, chunk_ukey, all_runtime_requirements);
  }

  for &entry_ukey in &entries {
    let mut all_runtime_requirements = RuntimeGlobals::default();
    let mut runtime_modules_to_add = Vec::<(ChunkUkey, Box<dyn RuntimeModule>)>::new();
    let entry = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&entry_ukey);

    for chunk_ukey in entry
      .get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      .iter()
    {
      let runtime_requirements =
        ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
      all_runtime_requirements.insert(*runtime_requirements);
    }

    let mut additional_runtime_modules = Vec::new();
    plugin_driver
      .compilation_hooks
      .additional_tree_runtime_requirements
      .call(
        compilation,
        &entry_ukey,
        &mut all_runtime_requirements,
        &mut additional_runtime_modules,
      )
      .await?;

    for module in additional_runtime_modules {
      let additional_runtime_requirements = module.additional_runtime_requirements(compilation);
      all_runtime_requirements.extend(additional_runtime_requirements);
      compilation.add_runtime_module(&entry_ukey, module)?;
    }

    let mut runtime_requirements_to_add = all_runtime_requirements;
    loop {
      let runtime_requirements_added = runtime_requirements_to_add;
      runtime_requirements_to_add = RuntimeGlobals::default();
      plugin_driver
        .compilation_hooks
        .runtime_requirement_in_tree
        .call(
          compilation,
          &entry_ukey,
          &all_runtime_requirements,
          &runtime_requirements_added,
          &mut runtime_requirements_to_add,
          &mut runtime_modules_to_add,
        )
        .await?;
      for runtime_module in &runtime_modules_to_add {
        let additional_runtime_requirements = runtime_module
          .1
          .additional_runtime_requirements(compilation);
        runtime_requirements_to_add.extend(additional_runtime_requirements);
      }
      runtime_requirements_to_add = runtime_requirements_to_add
        .difference(all_runtime_requirements.intersection(runtime_requirements_to_add));
      if runtime_requirements_to_add.is_empty() {
        break;
      }
      all_runtime_requirements.insert(runtime_requirements_to_add);
    }

    ChunkGraph::set_tree_runtime_requirements(compilation, entry_ukey, all_runtime_requirements);
    for (chunk_ukey, module) in runtime_modules_to_add {
      compilation.add_runtime_module(&chunk_ukey, module)?;
    }
  }

  let mut runtime_modules = std::mem::take(&mut compilation.runtime_modules);
  for entry_ukey in &entries {
    let runtime_module_ids = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_runtime_modules_iterable(entry_ukey)
      .copied()
      .collect::<Vec<_>>();
    for runtime_module_id in runtime_module_ids {
      plugin_driver
        .compilation_hooks
        .runtime_module
        .call(
          compilation,
          &runtime_module_id,
          entry_ukey,
          &mut runtime_modules,
        )
        .await?;
    }
  }
  compilation.runtime_modules = runtime_modules;

  Ok(())
}

async fn run_runtime_requirements_pass(compiler: &mut Compiler) -> Result<()> {
  compiler
    .cache
    .before_modules_runtime_requirements(&mut compiler.compilation)
    .await;
  compiler
    .cache
    .before_chunks_runtime_requirements(&mut compiler.compilation)
    .await;
  run_runtime_requirements_on_compilation(&mut compiler.compilation).await?;
  compiler
    .cache
    .after_modules_runtime_requirements(&compiler.compilation)
    .await;
  compiler
    .cache
    .after_chunks_runtime_requirements(&compiler.compilation)
    .await;
  Ok(())
}

async fn process_chunk_hash(
  compilation: &Compilation,
  chunk_ukey: ChunkUkey,
) -> Result<(rspack_hash::RspackHashDigest, ChunkContentHash)> {
  let mut hasher = RspackHash::from(&compilation.options.output);
  if let Some(chunk) = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(&chunk_ukey)
  {
    chunk.update_hash(&mut hasher, compilation);
  }

  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .chunk_hash
    .call(compilation, &chunk_ukey, &mut hasher)
    .await?;
  let chunk_hash = hasher.digest(&compilation.options.output.hash_digest);

  let mut content_hashes = FxHashMap::default();
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .content_hash
    .call(compilation, &chunk_ukey, &mut content_hashes)
    .await?;

  let content_hashes = content_hashes
    .into_iter()
    .map(|(source_type, mut content_hash)| {
      chunk_hash.hash(&mut content_hash);
      (
        source_type,
        content_hash.digest(&compilation.options.output.hash_digest),
      )
    })
    .collect();

  Ok((chunk_hash, content_hashes))
}

fn aggregate_full_hash(compilation: &mut Compilation) -> rspack_hash::RspackHashDigest {
  let mut compilation_hasher = RspackHash::from(&compilation.options.output);
  let mut chunks = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
    .collect::<Vec<_>>();
  chunks.sort_unstable_by_key(|chunk| chunk.ukey());

  for chunk in chunks {
    if let Some(hash) = chunk.hash(&compilation.chunk_hashes_artifact) {
      hash.hash(&mut compilation_hasher);
    }
    if let Some(content_hashes) = chunk.content_hash(&compilation.chunk_hashes_artifact) {
      let mut content_hash_entries = content_hashes.iter().collect::<Vec<_>>();
      content_hash_entries.sort_unstable_by_key(|(source_type, _)| *source_type);
      for (source_type, content_hash) in content_hash_entries {
        source_type.hash(&mut compilation_hasher);
        content_hash.hash(&mut compilation_hasher);
      }
    }
  }

  compilation.hot_index.hash(&mut compilation_hasher);
  let full_hash = compilation_hasher.digest(&compilation.options.output.hash_digest);
  compilation.hash = Some(full_hash.clone());
  full_hash
}

async fn compute_concatenated_module_codegen(
  compilation: &Compilation,
  concatenated_modules: &[rspack_core::ModuleIdentifier],
) -> Result<usize> {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let module_graph = compilation.get_module_graph();
  let mut jobs = Vec::new();

  for &module_identifier in concatenated_modules {
    let mut grouped_jobs = HashMap::<rspack_hash::RspackHashDigest, CodeGenerationJob>::new();
    for runtime in chunk_graph.get_module_runtimes_iter(module_identifier, chunk_by_ukey) {
      let hash = ChunkGraph::get_module_hash(compilation, module_identifier, runtime)
        .expect("concatenated module should have a module hash")
        .clone();
      let scope = compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .concatenation_scope
        .call(compilation, module_identifier)
        .await?;

      if let Some(job) = grouped_jobs.get_mut(&hash) {
        job.runtimes.push(runtime.clone());
      } else {
        grouped_jobs.insert(
          hash.clone(),
          CodeGenerationJob {
            module: module_identifier,
            hash,
            runtime: runtime.clone(),
            runtimes: vec![runtime.clone()],
            scope,
          },
        );
      }
    }
    jobs.extend(grouped_jobs.into_values());
  }

  let mut generated = 0;
  for job in jobs {
    let module = module_graph
      .module_by_identifier(&job.module)
      .expect("should have concatenated module");
    let mut runtime_template = compilation.runtime_template.create_module_code_template();
    let mut code_generation_context = ModuleCodeGenerationContext {
      compilation,
      runtime: Some(&job.runtime),
      concatenation_scope: job.scope.clone(),
      runtime_template: &mut runtime_template,
    };
    let mut code_generation_result = module.code_generation(&mut code_generation_context).await?;
    code_generation_result
      .runtime_requirements
      .extend(*runtime_template.runtime_requirements());
    code_generation_result.set_hash_for_concatenated_module(
      &job.hash,
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );
    black_box(code_generation_result);
    generated += 1;
  }

  Ok(generated)
}

async fn prepare_large_concatenation_case(
  groups: usize,
  modules_per_group: usize,
  fs: &MemoryFileSystem,
) {
  fs.create_dir_all("/src".into()).await.unwrap();
  let mut root_imports = Vec::with_capacity(groups);
  let mut root_values = Vec::with_capacity(groups);

  for group in 0..groups {
    let group_dir = format!("/src/group-{group}");
    fs.create_dir_all(group_dir.as_str().into()).await.unwrap();

    let mut group_imports = Vec::with_capacity(modules_per_group);
    let mut group_values = Vec::with_capacity(modules_per_group);

    for module in 0..modules_per_group {
      let file = format!("/src/group-{group}/module-{module}.js");
      let code = if module == 0 {
        format!("export const value = {group};")
      } else {
        format!(
          "import {{ value as prev }} from './module-{}.js'; export const value = prev + {};",
          module - 1,
          module
        )
      };
      fs.write(file.as_str().into(), code.as_bytes())
        .await
        .unwrap();
      group_imports.push(format!(
        "import {{ value as v{module} }} from './module-{module}.js';"
      ));
      group_values.push(format!("v{module}"));
    }

    let group_entry = format!(
      "{}\nexport default {};",
      group_imports.join("\n"),
      group_values.join(" + ")
    );
    fs.write(
      format!("/src/group-{group}/entry.js").as_str().into(),
      group_entry.as_bytes(),
    )
    .await
    .unwrap();

    root_imports.push(format!(
      "import g{group} from '/src/group-{group}/entry.js';"
    ));
    root_values.push(format!("g{group}"));
  }

  let entry = format!(
    "{}\nconsole.log({});",
    root_imports.join("\n"),
    root_values.join(" + ")
  );
  fs.write("/src/index.js".into(), entry.as_bytes())
    .await
    .unwrap();
}

async fn prepare_large_split_chunks_case(
  entry_count: usize,
  shared_modules: usize,
  fs: &MemoryFileSystem,
) {
  fs.create_dir_all("/src/entries".into()).await.unwrap();
  fs.create_dir_all("/src/shared".into()).await.unwrap();

  for module_index in 0..shared_modules {
    let code = format!("export default {module_index};");
    fs.write(
      format!("/src/shared/shared-{module_index}.js")
        .as_str()
        .into(),
      code.as_bytes(),
    )
    .await
    .unwrap();
  }

  let rotating_pool = shared_modules - SPLIT_CHUNKS_COMMON_MODULES;
  for entry_index in 0..entry_count {
    let mut imports = Vec::with_capacity(SPLIT_CHUNKS_WINDOW + SPLIT_CHUNKS_COMMON_MODULES);
    let mut values = Vec::with_capacity(SPLIT_CHUNKS_WINDOW + SPLIT_CHUNKS_COMMON_MODULES);

    for offset in 0..SPLIT_CHUNKS_WINDOW {
      let module_index = (entry_index * 7 + offset) % rotating_pool;
      imports.push(format!(
        "import shared_{module_index} from '/src/shared/shared-{module_index}.js';"
      ));
      values.push(format!("shared_{module_index}"));
    }

    for offset in 0..SPLIT_CHUNKS_COMMON_MODULES {
      let module_index = rotating_pool + offset;
      imports.push(format!(
        "import shared_{module_index} from '/src/shared/shared-{module_index}.js';"
      ));
      values.push(format!("shared_{module_index}"));
    }

    let source = format!(
      "{}\nconsole.log({});",
      imports.join("\n"),
      values.join(" + ")
    );
    fs.write(
      format!("/src/entries/entry-{entry_index}.js")
        .as_str()
        .into(),
      source.as_bytes(),
    )
    .await
    .unwrap();
  }
}

fn count_concatenated_modules(compilation: &Compilation) -> usize {
  compilation
    .get_module_graph()
    .modules()
    .filter(|(_, module)| module.as_concatenated_module().is_some())
    .count()
}

fn count_assigned_export_used_names(compilation: &Compilation) -> usize {
  compilation
    .get_module_graph()
    .modules()
    .map(|(module_identifier, _)| {
      compilation
        .exports_info_artifact
        .get_exports_info_data(module_identifier)
        .exports()
        .values()
        .filter(|export_info| export_info.used_name().is_some())
        .count()
    })
    .sum()
}

fn assert_no_compilation_errors(compilation: &Compilation, context: &str) {
  assert!(
    compilation.get_errors().next().is_none(),
    "{context} should not produce compilation errors"
  );
}

fn create_split_chunks_plugin() -> SplitChunksPlugin {
  let js_zero_sizes = SplitChunkSizes::with_initial_value(&[SourceType::JavaScript], 0.0);

  SplitChunksPlugin::new(PluginOptions {
    cache_groups: vec![CacheGroup {
      key: "shared-modules".to_string(),
      chunk_filter: create_all_chunk_filter(),
      test: CacheGroupTest::String("/src/shared/".to_string()),
      r#type: create_default_module_type_filter(),
      layer: create_default_module_layer_filter(),
      name: ChunkNameGetter::Disabled,
      priority: 0.0,
      min_size: js_zero_sizes.clone(),
      min_size_reduction: js_zero_sizes.clone(),
      enforce_size_threshold: SplitChunkSizes::default(),
      reuse_existing_chunk: false,
      min_chunks: 2,
      id_hint: "shared-modules".to_string(),
      max_initial_requests: f64::INFINITY,
      max_async_requests: f64::INFINITY,
      max_async_size: SplitChunkSizes::default(),
      max_initial_size: SplitChunkSizes::default(),
      filename: None,
      automatic_name_delimiter: DEFAULT_DELIMITER.to_string(),
      used_exports: false,
    }],
    fallback_cache_group: FallbackCacheGroup {
      chunks_filter: create_all_chunk_filter(),
      min_size: js_zero_sizes,
      max_async_size: SplitChunkSizes::default(),
      max_initial_size: SplitChunkSizes::default(),
      automatic_name_delimiter: DEFAULT_DELIMITER.to_string(),
    },
    hide_path_info: Some(true),
  })
}

criterion_group!(compilation_stages, compilation_stages_benchmark);
