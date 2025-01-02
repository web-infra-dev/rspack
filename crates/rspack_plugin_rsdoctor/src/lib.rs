use std::sync::Arc;
use std::{borrow::Cow, sync::atomic::AtomicUsize};

use async_trait::async_trait;
use rspack_collections::Identifier;
use rspack_core::{
  concatenated_module, ApplyContext, Chunk, Compilation, CompilationFinishModules,
  CompilationOptimizeChunkModules, CompilationOptimizeModules, CompilerOptions, DependencyType,
  Module, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

type HandlerFn = Arc<dyn Fn(f64, String, Vec<String>) -> Result<()> + Send + Sync>;

#[derive(Default)]
pub struct RsdoctorPluginOptions {
  module_graph_cb: Option<HandlerFn>,
  chunk_graph_cb: Option<HandlerFn>,
}

impl std::fmt::Debug for RsdoctorPluginOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RsdoctorPluginOptions")
      .field("module_graph_cb", &self.module_graph_cb.is_some())
      .field("chunk_graph_cb", &self.chunk_graph_cb.is_some())
      .finish()
  }
}

enum ModuleKind {
  Normal,
  Concatenated,
}

type ModuleUkey = usize;
type DependencyUkey = usize;
type ChunkUkey = usize;

struct RsdoctorModule {
  ukey: ModuleUkey,
  identifier: Identifier,
  path: String,
  is_entry: bool,
  kind: ModuleKind,
  layer: Option<String>,
  dependencies: HashSet<DependencyUkey>,
  imported: HashSet<ModuleUkey>,
  modules: HashSet<ModuleUkey>,
}

struct RsdoctorDependency {
  ukey: DependencyUkey,
  kind: DependencyType,
  request: String,
  module: ModuleUkey,
  dependency: ModuleUkey,
}

#[plugin]
#[derive(Debug)]
pub struct RsdoctorPlugin {
  pub options: RsdoctorPluginOptions,
}

#[plugin_hook(CompilationOptimizeChunkModules for RsdoctorPlugin)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let module_ukey_counter = Arc::new(AtomicUsize::new(0));
  let dependency_ukey_counter = Arc::new(AtomicUsize::new(0));

  let mut rsd_modules = HashMap::default();
  let mut rsd_dependencies = HashMap::default();

  // let module_id_map = HashMap::default();
  // let module_ident_map = HashMap::default();
  // let layer_set = HashSet::default();

  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;
  let chunk_by_ukey = &compilation.chunk_by_ukey;

  // 1. collect modules
  let modules = module_graph.modules();
  for (module_id, module) in modules.iter() {
    let ukey = module_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let depth = module_graph.get_depth(module_id);
    let path = if let Some(nfc) = module.name_for_condition() {
      nfc.to_string()
    } else {
      module
        .readable_identifier(&compilation.options.context)
        .to_string()
    };
    let is_concatenated = module.as_concatenated_module().is_some();
    rsd_modules.insert(
      module_id,
      RsdoctorModule {
        ukey,
        identifier: module.identifier(),
        path,
        is_entry: depth.is_some_and(|d| d == 0),
        kind: if is_concatenated {
          ModuleKind::Concatenated
        } else {
          ModuleKind::Normal
        },
        layer: module.get_layer().map(|layer| layer.to_string()),
        dependencies: HashSet::default(),
        imported: HashSet::default(),
        modules: HashSet::default(),
      },
    );
  }

  // 2. concatenate modules
  for (module_id, module) in module_graph.modules().iter() {
    let children = module.as_concatenated_module().map(|concatenated_module| {
      concatenated_module
        .get_modules()
        .iter()
        .filter_map(|m| rsd_modules.get(&m.id).map(|m| m.ukey))
        .collect::<HashSet<_>>()
    });

    if let Some(rsd_module) = rsd_modules.get_mut(module_id) {
      rsd_module.modules.extend(children.unwrap_or_default());
    }
  }

  // 3. collect dependencies
  for (module_id, _) in module_graph.modules().iter() {
    let Some(rsd_module_ukey) = rsd_modules.get(module_id).map(|m| m.ukey) else {
      continue;
    };
    let dependencies = module_graph
      .get_outgoing_connections(module_id)
      .filter_map(|conn| {
        let Some(dep) = module_graph.dependency_by_id(&conn.dependency_id) else {
          return None;
        };
        if let (Some(dep), Some(dep_module)) = (
          dep.as_module_dependency(),
          module_graph
            .module_identifier_by_dependency_id(&conn.dependency_id)
            .and_then(|mid| rsd_modules.get(mid)),
        ) {
          let dep_ukey = dependency_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
          rsd_dependencies
            .entry(conn.dependency_id)
            .or_insert(RsdoctorDependency {
              ukey: dep_ukey,
              kind: *dep.dependency_type(),
              request: dep.user_request().into(),
              module: rsd_module_ukey,
              dependency: dep_module.ukey,
            });

          return Some((dep_ukey, dep_module.identifier));
        }
        None
      })
      .collect::<HashSet<_>>();

    for (dep_id, dep_module_ukey) in dependencies {
      if let Some(rsd_module) = rsd_modules.get_mut(&dep_module_ukey) {
        rsd_module.imported.insert(rsd_module_ukey);
      }
      if let Some(rsd_module) = rsd_modules.get_mut(module_id) {
        rsd_module.dependencies.insert(dep_id);
      }
    }
  }

  Ok(None)
}

#[async_trait]
impl Plugin for RsdoctorPlugin {
  fn name(&self) -> &'static str {
    "rsdoctor"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));
    Ok(())
  }
}
