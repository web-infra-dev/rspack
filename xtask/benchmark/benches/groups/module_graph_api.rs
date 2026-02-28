#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use criterion::criterion_group;
use rspack::builder::Builder as _;
use rspack_benchmark::Criterion;
use rspack_core::{
  Compiler, Optimization, build_module_graph::build_module_graph_pass,
  incremental::IncrementalOptions,
};
use rspack_fs::{MemoryFileSystem, WritableFileSystem};
use rspack_tasks::within_compiler_context_for_testing_sync;
use tokio::runtime::Builder;

use crate::groups::build_chunk_graph::{NUM_MODULES, prepare_large_code_splitting_case};

pub fn module_graph_api_benchmark(c: &mut Criterion) {
  within_compiler_context_for_testing_sync(|| {
    module_graph_api_benchmark_inner(c);
  })
}

pub fn module_graph_api_benchmark_inner(c: &mut Criterion) {
  let rt = Builder::new_multi_thread()
    .build()
    .expect("should not fail to build tokio runtime");
  let _guard = rt.enter();

  let fs = Arc::new(MemoryFileSystem::default());
  let random_table =
    serde_json::from_str::<Vec<Vec<usize>>>(include_str!("../build_chunk_graph/random_table.json"))
      .expect("should not fail to parse random table json");
  let mut compiler = Compiler::builder()
    .context("/")
    .entry("main", "/src/dynamic-0.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs.clone())
    .optimization(Optimization::builder().remove_available_modules(true))
    .incremental(IncrementalOptions::empty_passes())
    .build()
    .unwrap();

  rt.block_on(async {
    fs.create_dir_all("/src".into())
      .await
      .expect("should not fail to create dir");
    prepare_large_code_splitting_case(NUM_MODULES, &random_table, &fs).await;

    let mut compilation_params = compiler.new_compilation_params();
    compiler
      .plugin_driver
      .compiler_hooks
      .this_compilation
      .call(&mut compiler.compilation, &mut compilation_params)
      .await
      .unwrap();
    compiler
      .plugin_driver
      .compiler_hooks
      .compilation
      .call(&mut compiler.compilation, &mut compilation_params)
      .await
      .unwrap();
    compiler
      .plugin_driver
      .compiler_hooks
      .make
      .call(&mut compiler.compilation)
      .await
      .unwrap();
    build_module_graph_pass(&mut compiler.compilation)
      .await
      .unwrap();
  });

  assert!(
    compiler.compilation.get_errors().next().is_none(),
    "module_graph_api benchmark setup should not produce compilation errors"
  );

  let dependency_ids = {
    let module_graph = compiler.compilation.get_module_graph();
    module_graph
      .dependencies()
      .map(|(dependency_id, _)| *dependency_id)
      .collect::<Vec<_>>()
  };
  let module_ids = {
    let module_graph = compiler.compilation.get_module_graph();
    module_graph.modules_keys().copied().collect::<Vec<_>>()
  };

  c.bench_function("rust@module_graph_api", |b| {
    b.iter(|| {
      let module_graph = compiler.compilation.get_module_graph();

      for dependency_id in &dependency_ids {
        let _ = module_graph.connection_by_dependency_id(dependency_id);
        let _ = module_graph.get_module_by_dependency_id(dependency_id);
        let _ = module_graph.get_resolved_module(dependency_id);
      }

      for module_id in &module_ids {
        for _ in module_graph.get_outgoing_connections(module_id) {}
        for _ in module_graph.get_ordered_outgoing_connections(module_id) {}
        for _ in module_graph.get_incoming_connections(module_id) {}
      }
    });
  });
}

criterion_group!(module_graph_api, module_graph_api_benchmark);
