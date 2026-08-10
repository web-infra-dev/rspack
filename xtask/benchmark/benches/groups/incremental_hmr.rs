#![allow(clippy::disallowed_methods, clippy::unwrap_used)]

use std::{
  cell::{Cell, RefCell},
  fs,
  path::{Path, PathBuf},
  sync::Arc,
};

use criterion::BatchSize;
use rspack::builder::{Builder as _, CompilerBuilder};
use rspack_benchmark::Criterion;
use rspack_core::{
  CacheOptions, Compiler, Experiments, Mode, Optimization, OutputOptions, PluginExt,
  incremental::{IncrementalOptions, IncrementalPasses},
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem};
use rspack_paths::ArcPath;
use rspack_plugin_copy::{CopyGlobOptions, CopyPattern, CopyRspackPlugin, ToOption, ToType};
use rspack_plugin_hmr::HotModuleReplacementPlugin;
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherIgnored, FsWatcherOptions,
};
use rustc_hash::{FxHashSet, FxHashSet as HashSet};

use super::diagnostics::assert_no_compilation_errors;

const ROUTE_COUNT: usize = 256;
const LEAVES_PER_ROUTE: usize = 8;
const CSS_GROUP_SIZE: usize = 8;
const COPY_ASSET_COUNT: usize = 64;
const CHANGED_MODULE: &str = "src/routes/route-0-leaf-0.js";
const CHANGED_STYLE: &str = "src/styles/group-0.css";

pub fn incremental_hmr_benchmark(c: &mut Criterion) {
  let rt = rspack_benchmark::build_tokio_rt();
  let workspace = prepare_workspace();
  let compiler_context = Arc::new(CompilerContext::new());
  let mut compiler = within_compiler_context_sync(compiler_context.clone(), || {
    incremental_compiler(&workspace).build().unwrap()
  });

  rt.block_on(within_compiler_context(compiler_context.clone(), async {
    compiler.run().await.unwrap();
  }));
  assert_no_compilation_errors(&compiler.compilation, "incremental HMR benchmark setup");

  let compiler = RefCell::new(compiler);
  let changed_module = workspace.join(CHANGED_MODULE);
  let module_revision = Cell::new(0usize);

  c.bench_function("rust@incremental_hmr@stable_graph_leaf_edit", |b| {
    b.iter_batched(
      || {
        let next_revision = module_revision.get().wrapping_add(1);
        module_revision.set(next_revision);
        fs::write(
          &changed_module,
          format!("export const value = {next_revision};\n"),
        )
        .unwrap();

        let mut changed_files = FxHashSet::default();
        changed_files.insert(changed_module.to_string_lossy().into_owned());
        changed_files
      },
      |changed_files| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(within_compiler_context(compiler_context.clone(), async {
          compiler
            .rebuild(changed_files, FxHashSet::default())
            .await
            .unwrap();
        }));
        assert_no_compilation_errors(&compiler.compilation, "incremental HMR benchmark rebuild");
      },
      BatchSize::PerIteration,
    );
  });

  let changed_style = workspace.join(CHANGED_STYLE);
  let style_revision = Cell::new(0usize);
  c.bench_function("rust@incremental_hmr@css_leaf_edit", |b| {
    b.iter_batched(
      || {
        let next_revision = style_revision.get().wrapping_add(1);
        style_revision.set(next_revision);
        fs::write(
          &changed_style,
          format!(".group-0 {{ --revision: {next_revision}; }}\n"),
        )
        .unwrap();

        let mut changed_files = FxHashSet::default();
        changed_files.insert(changed_style.to_string_lossy().into_owned());
        changed_files
      },
      |changed_files| {
        let mut compiler = compiler.borrow_mut();
        rt.block_on(within_compiler_context(compiler_context.clone(), async {
          compiler
            .rebuild(changed_files, FxHashSet::default())
            .await
            .unwrap();
        }));
        assert_no_compilation_errors(
          &compiler.compilation,
          "incremental HMR CSS benchmark rebuild",
        );
      },
      BatchSize::PerIteration,
    );
  });

  let watched_file = changed_module.to_string_lossy().into_owned();
  c.bench_function("rust@incremental_hmr@native_watcher_cycle", |b| {
    b.iter(|| {
      rt.block_on(async {
        let watcher = FsWatcher::new(
          FsWatcherOptions {
            aggregate_timeout: Some(0),
            ..Default::default()
          },
          FsWatcherIgnored::None,
        );
        watcher
          .watch(
            (
              [ArcPath::from(watched_file.as_str())].into_iter(),
              std::iter::empty(),
            ),
            (std::iter::empty(), std::iter::empty()),
            (std::iter::empty(), std::iter::empty()),
            std::time::SystemTime::UNIX_EPOCH,
            Box::new(NoopAggregateHandler),
            Box::new(NoopEventHandler),
          )
          .await;
        for _ in 0..32 {
          watcher.trigger_event(&ArcPath::from(watched_file.as_str()), FsEventKind::Change);
        }
        watcher.pause().unwrap();
        watcher.close().await.unwrap();
      });
    });
  });
}

fn incremental_compiler(workspace: &Path) -> CompilerBuilder {
  let mut builder = Compiler::builder();
  builder
    .context(workspace.to_string_lossy().into_owned())
    .entry("main", "./src/index.js")
    .mode(Mode::Development)
    .cache(CacheOptions::Disabled)
    .optimization(Optimization::builder().minimize(false))
    .experiments(Experiments::builder().css(true))
    .incremental(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::all(),
    })
    .output(OutputOptions::builder().compare_before_emit(false))
    .input_filesystem(Arc::new(NativeFileSystem::new(false)))
    .output_filesystem(Arc::new(MemoryFileSystem::default()))
    .plugin(HotModuleReplacementPlugin::default().boxed())
    .plugin(
      CopyRspackPlugin::new(vec![CopyPattern {
        from: "assets".to_string(),
        to: Some(ToOption::String("copied".to_string())),
        context: Some(workspace.to_string_lossy().into_owned().into()),
        to_type: Some(ToType::Dir),
        no_error_on_missing: false,
        info: None,
        force: false,
        priority: 0,
        glob_options: CopyGlobOptions {
          case_sensitive_match: Some(true),
          dot: Some(false),
          ignore: None,
        },
        copy_permissions: Some(false),
        transform_fn: None,
        cache: Some(true),
      }])
      .boxed(),
    );
  builder
}

fn prepare_workspace() -> PathBuf {
  let workspace = std::env::temp_dir().join(format!(
    "rspack-codspeed-incremental-hmr-{}",
    std::process::id()
  ));
  let src = workspace.join("src");
  let routes = src.join("routes");
  let styles = src.join("styles");
  let assets = workspace.join("assets");
  fs::create_dir_all(&routes).unwrap();
  fs::create_dir_all(&styles).unwrap();
  fs::create_dir_all(&assets).unwrap();

  let mut index =
    String::from("if (module.hot) module.hot.accept();\nimport './styles/global.css';\n");
  for route in 0..ROUTE_COUNT {
    index.push_str(&format!(
      "export const route{route} = () => import(/* webpackChunkName: \"route-{route}\" */ './routes/route-{route}.js');\n"
    ));

    let mut route_source = format!("import '../styles/group-{}.css';\n", route / CSS_GROUP_SIZE);
    for leaf in 0..LEAVES_PER_ROUTE {
      route_source.push_str(&format!(
        "import {{ value as value{leaf} }} from './route-{route}-leaf-{leaf}.js';\n"
      ));
      fs::write(
        routes.join(format!("route-{route}-leaf-{leaf}.js")),
        format!(
          "export const value = {};\n",
          route * LEAVES_PER_ROUTE + leaf
        ),
      )
      .unwrap();
    }
    route_source.push_str("if (module.hot) module.hot.accept();\n");
    route_source.push_str("export default [");
    route_source.push_str(
      &(0..LEAVES_PER_ROUTE)
        .map(|leaf| format!("value{leaf}"))
        .collect::<Vec<_>>()
        .join(","),
    );
    route_source.push_str("];\n");
    fs::write(routes.join(format!("route-{route}.js")), route_source).unwrap();
  }
  fs::write(src.join("index.js"), index).unwrap();
  fs::write(
    styles.join("global.css"),
    "body { color: #123456; background: #fafafa; }\n",
  )
  .unwrap();
  for group in 0..ROUTE_COUNT / CSS_GROUP_SIZE {
    fs::write(
      styles.join(format!("group-{group}.css")),
      format!(".group-{group} {{ --group: {group}; }}\n"),
    )
    .unwrap();
  }
  for asset in 0..COPY_ASSET_COUNT {
    fs::write(
      assets.join(format!("asset-{asset}.txt")),
      format!("static asset {asset}\n"),
    )
    .unwrap();
  }

  workspace
}

struct NoopAggregateHandler;

impl EventAggregateHandler for NoopAggregateHandler {
  fn on_event_handle(&self, _changed_files: HashSet<String>, _deleted_files: HashSet<String>) {}
}

struct NoopEventHandler;

impl EventHandler for NoopEventHandler {}
