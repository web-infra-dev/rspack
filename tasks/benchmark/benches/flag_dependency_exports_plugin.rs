#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use criterion::criterion_group;
use rspack::builder::Builder as _;
use rspack_benchmark::Criterion;
use rspack_collections::IdentifierSet;
use rspack_core::{
  fast_set, incremental::IncrementalPasses, Compilation, Compiler, Experiments, Optimization,
};
use rspack_fs::{MemoryFileSystem, WritableFileSystem};
use rspack_plugin_javascript::FlagDependencyExportsState;
use tokio::runtime::Builder;

static NUM_MODULES: usize = 10000;

async fn prepare_large_code_splitting_case(
  num: usize,
  random_table: &Vec<Vec<usize>>,
  fs: &MemoryFileSystem,
) {
  let mut ctx: Vec<(String, String)> = vec![];
  gen_dynamic_module(num, 0, random_table, &mut ctx);

  fs.create_dir_all("/src".into()).await.unwrap();
  fs.create_dir_all("/src/leaves".into()).await.unwrap();

  for (name, code) in ctx {
    fs.write(name.as_str().into(), code.as_bytes())
      .await
      .unwrap();
  }
}

fn gen_static_leaf_module(index: usize, ctx: &mut Vec<(String, String)>) {
  let code = "
function Navbar({ show }) {
	return (
    {show}
	)
}
export default Navbar";

  ctx.push((
    format!("/src/leaves/Component-{}.js", index)
      .as_str()
      .into(),
    code.to_string(),
  ));
}

fn gen_dynamic_module(
  num: usize,
  index: usize,
  random_table: &Vec<Vec<usize>>,
  ctx: &mut Vec<(String, String)>,
) -> bool {
  if index >= num {
    return false;
  }

  let mut access = vec![];
  let mut static_imports = vec![];
  let mut dynamic_imports = vec![];
  let mut reuse = vec![];

  for i in index..index + 10 {
    static_imports.push(format!(
      "import Comp{} from '/src/leaves/Component-{}.js'",
      i, i,
    ));
    gen_static_leaf_module(i, ctx);
    access.push(format!("Comp{}", i));
  }

  let depth = index / 10;
  for random in random_table[depth].iter() {
    reuse.push(format!(
      "import Comp{} from '/src/leaves/Component-{}.js'",
      random, random,
    ));
    access.push(format!("Comp{}", random));
  }

  if gen_dynamic_module(num, index + 10, random_table, ctx) {
    dynamic_imports.push(format!("import('/src/dynamic-{}.js')", depth + 1));
  }

  let code = format!(
    "{}\n{}\n{}\n{};export default {};",
    static_imports.join("\n"),
    reuse.join("\n"),
    access.join("\n"),
    dynamic_imports.join("\n"),
    depth
  );

  ctx.push((format!("/src/dynamic-{}.js", depth).as_str().into(), code));
  true
}

pub fn flag_dependency_exports_plugin_benchmark(c: &mut Criterion) {
  let rt = Builder::new_multi_thread()
    .build()
    .expect("should not fail to build tokio runtime");

  let fs = Arc::new(MemoryFileSystem::default());
  let random_table =
    serde_json::from_str::<Vec<Vec<usize>>>(include_str!("build_chunk_graph/random_table.json"))
      .expect("should not fail to parse random table json");

  let mut compiler = Compiler::builder()
    .context("/")
    .entry("main", "/src/dynamic-0.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs.clone())
    .optimization(Optimization::builder().remove_available_modules(true))
    .experiments(Experiments::builder().incremental(IncrementalPasses::empty()))
    .build();

  fast_set(
    &mut compiler.compilation,
    Compilation::new(
      compiler.options.clone(),
      compiler.plugin_driver.clone(),
      compiler.buildtime_plugin_driver.clone(),
      compiler.resolver_factory.clone(),
      compiler.loader_resolver_factory.clone(),
      None,
      compiler.cache.clone(),
      compiler.old_cache.clone(),
      Some(Default::default()),
      Default::default(),
      Default::default(),
      compiler.input_filesystem.clone(),
      compiler.intermediate_filesystem.clone(),
      compiler.output_filesystem.clone(),
    ),
  );

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
    compiler.compilation.make().await.unwrap();

    while matches!(
      compiler
        .plugin_driver
        .compilation_hooks
        .optimize_dependencies
        .call(&mut compiler.compilation)
        .unwrap(),
      Some(true)
    ) {}

    compiler
      .compilation
      .finish(compiler.plugin_driver.clone())
      .await
      .unwrap();
  });

  assert!(compiler.compilation.get_errors().next().is_none());

  c.bench_function("flag_dependency_exports", |b| {
    b.iter_with_setup_wrapper(|runner| {
      let logger = compiler
        .compilation
        .get_logger("rspack.FlagDependencyExportsPlugin");

      let module_graph = &mut compiler.compilation.get_module_graph_mut();

      let module_ids = module_graph
        .modules()
        .keys()
        .copied()
        .collect::<IdentifierSet>();

      let mut state = FlagDependencyExportsState::new(module_graph, logger);

      runner.run(|| {
        state.apply(module_ids);
      });
    });
  });
}

criterion_group!(
  flag_dependency_exports_plugin,
  flag_dependency_exports_plugin_benchmark
);
