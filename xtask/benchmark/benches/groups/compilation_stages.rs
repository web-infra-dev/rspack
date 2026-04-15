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
  AssignRuntimeIdsPass, AsyncModulesArtifact, CacheOptions, ChunkByUkey, ChunkContentHash,
  ChunkGraph, ChunkNamedIdArtifact, ChunkUkey, CodeGenerationJob, CodeGenerationPass, Compilation,
  CompilationAsset, CompilationAssets, Compiler, CreateChunkAssetsPass, CreateHashPass,
  CreateModuleAssetsPass, CreateModuleHashesPass, DEFAULT_DELIMITER, MangleExportsOption, Mode,
  ModuleCodeGenerationContext, ModuleIdsArtifact, Optimization, OptimizeCodeGenerationPass,
  OutputOptions, ProcessAssetsPass, RuntimeRequirementsPass, SideEffectsOptimizeArtifact,
  SourceType, UsedExportsOption, build_chunk_graph,
  build_module_graph::{build_module_graph_pass, finish_build_module_graph},
  cache::Cache,
  incremental::IncrementalOptions,
  pass::PassExt,
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
    run_compiler_pass(&OptimizeCodeGenerationPass, &mut compiler)
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
          run_compiler_pass(&OptimizeCodeGenerationPass, &mut compiler)
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
    prepare_for_runtime_requirements(&mut compiler)
      .await
      .unwrap();
    run_runtime_requirements_pass(&mut compiler).await.unwrap();
  });

  assert_no_compilation_errors(&compiler.compilation, "create_full_hash setup");
  let initial_code_generated_modules = compiler.compilation.code_generated_modules.clone();

  rt.block_on(async {
    run_create_hash_pass(&mut compiler).await.unwrap();
  });
  assert!(
    !compiler.compilation.chunk_hashes_artifact.is_empty(),
    "create_full_hash setup should prepare chunk hashes"
  );
  assert!(
    compiler.compilation.hash.is_some(),
    "create_full_hash setup should set the compilation hash"
  );
  compiler.compilation.chunk_hashes_artifact.clear();
  compiler.compilation.runtime_modules_hash.clear();
  compiler
    .compilation
    .runtime_modules_code_generation_source
    .clear();
  compiler.compilation.hash = None;
  compiler.compilation.code_generated_modules = initial_code_generated_modules.clone();

  let compiler = RefCell::new(compiler);

  c.bench_function("rust@create_full_hash", |b| {
    b.iter_batched_ref(
      || {
        let mut compiler = compiler.borrow_mut();
        compiler.compilation.chunk_hashes_artifact.clear();
        compiler.compilation.runtime_modules_hash.clear();
        compiler
          .compilation
          .runtime_modules_code_generation_source
          .clear();
        compiler.compilation.hash = None;
        compiler.compilation.code_generated_modules = initial_code_generated_modules.clone();
      },
      |_| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(async {
          run_create_hash_pass(&mut compiler).await.unwrap();
        });
        black_box((
          !compiler.compilation.chunk_hashes_artifact.is_empty(),
          compiler.compilation.hash.is_some(),
          compiler
            .compilation
            .runtime_modules_code_generation_source
            .len(),
        ));
      },
      BatchSize::PerIteration,
    );
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
  run_compiler_pass(&AssignRuntimeIdsPass, compiler).await?;
  Ok(())
}

async fn prepare_for_module_hashes(compiler: &mut Compiler) -> Result<()> {
  prepare_for_chunk_ids(compiler).await?;
  run_chunk_ids_on_compilation(&mut compiler.compilation).await?;
  Ok(())
}

async fn prepare_for_runtime_requirements(compiler: &mut Compiler) -> Result<()> {
  prepare_for_optimize_code_generation(compiler).await?;
  run_compiler_pass(&OptimizeCodeGenerationPass, compiler).await?;
  run_create_module_hashes_pass(compiler).await?;
  run_code_generation_pass(compiler).await?;
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

async fn run_create_module_hashes_pass(compiler: &mut Compiler) -> Result<()> {
  run_compiler_pass(&CreateModuleHashesPass, compiler).await
}

async fn run_code_generation_pass(compiler: &mut Compiler) -> Result<()> {
  run_compiler_pass(&CodeGenerationPass, compiler).await
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
  let current_asset_names = compilation.assets().keys().cloned().collect::<Vec<_>>();
  for asset_name in current_asset_names {
    compilation.delete_asset(&asset_name);
  }
  for (asset_name, asset) in snapshot.assets.clone() {
    compilation.emit_asset(asset_name, asset);
  }
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

async fn run_compiler_pass<P: PassExt>(pass: &P, compiler: &mut Compiler) -> Result<()> {
  let compilation = &mut compiler.compilation;
  let cache = &mut compiler.cache;
  pass.run(compilation, &mut **cache).await
}

async fn run_create_hash_pass(compiler: &mut Compiler) -> Result<()> {
  run_compiler_pass(&CreateHashPass, compiler).await
}

async fn run_create_module_assets_pass(compilation: &mut Compilation) -> Result<()> {
  let mut noop_cache = NoopCache;
  CreateModuleAssetsPass
    .run(compilation, &mut noop_cache)
    .await
}

fn seed_module_assets(compilation: &mut Compilation) -> usize {
  let mut module_identifiers = compilation
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
    .collect::<Vec<_>>();
  module_identifiers.sort_unstable();
  module_identifiers.truncate(MODULE_ASSET_SEED_COUNT);

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
  run_compiler_pass(&CreateChunkAssetsPass, compiler).await
}

async fn run_process_assets_pass(compilation: &mut Compilation) -> Result<()> {
  let mut noop_cache = NoopCache;
  ProcessAssetsPass.run(compilation, &mut noop_cache).await
}

#[derive(Debug, Default)]
struct NoopCache;

#[async_trait::async_trait]
impl Cache for NoopCache {}

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

async fn run_runtime_requirements_pass(compiler: &mut Compiler) -> Result<()> {
  run_compiler_pass(&RuntimeRequirementsPass, compiler).await
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
