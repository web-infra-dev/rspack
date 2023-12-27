use std::cmp::Ordering;
use std::os::unix::prelude::OpenOptionsExt;
use std::sync::Arc;

use rspack_core::{
  BoxDependency, Compilation, ExportInfoProvided, ExtendedReferencedExport, Logger,
  ModuleIdentifier, OptimizeChunksArgs, Plugin, ProvidedExports, RuntimeSpec,
  WrappedModuleIdentifier,
};
use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportSideEffectDependency,
  HarmonyImportSpecifierDependency,
};
fn format_bailout_reason(msg: &str) -> String {
  format!("ModuleConcatenation bailout: {}", msg)
}

#[derive(Clone)]
enum Warning {
  Id(ModuleIdentifier),
  Problem(Arc<dyn Fn(String) -> String>),
}
struct ConcatConfiguration<'a> {
  root_module: ModuleIdentifier,
  runtime: &'a RuntimeSpec,
  modules: HashSet<ModuleIdentifier>,
  warnings: HashMap<ModuleIdentifier, Warning>,
}

#[allow(unused)]
impl<'a> ConcatConfiguration<'a> {
  fn new(root_module: ModuleIdentifier, runtime: &'a RuntimeSpec) -> Self {
    let mut modules = HashSet::default();
    modules.insert(root_module);

    ConcatConfiguration {
      root_module,
      runtime,
      modules,
      warnings: HashMap::default(),
    }
  }

  fn add(&mut self, module: ModuleIdentifier) {
    self.modules.insert(module);
  }

  fn has(&self, module: &ModuleIdentifier) -> bool {
    self.modules.contains(module)
  }

  fn is_empty(&self) -> bool {
    self.modules.len() == 1
  }

  fn add_warning(&mut self, module: ModuleIdentifier, problem: Warning) {
    self.warnings.insert(module, problem);
  }

  fn get_warnings_sorted(&self) -> HashMap<ModuleIdentifier, Warning> {
    let mut sorted_warnings: Vec<_> = self.warnings.clone().into_iter().collect();
    sorted_warnings.sort_by(|a, b| a.0.cmp(&b.0));
    sorted_warnings.into_iter().collect()
  }

  fn get_modules(&self) -> &HashSet<ModuleIdentifier> {
    &self.modules
  }

  fn snapshot(&self) -> usize {
    self.modules.len()
  }

  fn rollback(&mut self, mut snapshot: usize) {
    let modules = &mut self.modules;
    modules.retain(|_| {
      if snapshot == 0 {
        false
      } else {
        snapshot -= 1;
        true
      }
    });
  }
}

#[derive(Debug)]
struct ModuleConcatenationPlugin;

impl ModuleConcatenationPlugin {
  fn get_imports(
    compilation: &Compilation,
    mi: WrappedModuleIdentifier,
    runtime: RuntimeSpec,
  ) -> HashSet<ModuleIdentifier> {
    let mg = &compilation.module_graph;
    let mut set = HashSet::default();
    let module = mg.module_by_identifier(&mi).expect("should have module");
    for d in module.get_dependencies() {
      let dep = d.get_dependency(mg);
      let is_harmony_import_like = extends_harmony_dep(dep);
      if !is_harmony_import_like {
        continue;
      }
      let Some(con) = mg.connection_by_dependency(d) else {
        continue;
      };
      if !con.is_target_active(mg, Some(&runtime)) {
        continue;
      }
      // SAFETY: because it is extends harmony dep, we can ensure the dep has been
      // implemented ModuleDepdency Trait.
      let module_dep = dep.as_module_dependency().expect("should be module dep");
      let imported_names = module_dep.get_referenced_exports(mg, None);
      if imported_names.iter().all(|item| match item {
        ExtendedReferencedExport::Array(arr) => !arr.is_empty(),
        ExtendedReferencedExport::Export(export) => !export.name.is_empty(),
      }) || matches!(mg.get_provided_exports(*mi), ProvidedExports::Vec(_))
      {
        set.insert(con.module_identifier);
      }
    }
    set
  }
}

#[async_trait::async_trait]
impl Plugin for ModuleConcatenationPlugin {
  async fn optimize_chunk_modules(&self, args: OptimizeChunksArgs<'_>) -> Result<()> {
    let OptimizeChunksArgs { compilation } = args;
    let logger = compilation.get_logger("rspack.ModuleConcatenationPlugin");
    let mut relevant_modules = vec![];
    let mut possible_inners = HashSet::default();
    let start = logger.time("select relevant modules");
    let module_id_list = compilation
      .module_graph
      .module_graph_modules()
      .keys()
      .copied()
      .map(WrappedModuleIdentifier::from)
      .collect::<Vec<_>>();
    let mg = &compilation.module_graph;
    let Compilation {
      module_graph: mg,
      chunk_graph,
      ..
    } = compilation;
    for module_id in module_id_list {
      let mut can_be_root = true;
      let mut can_be_inner = true;
      let mgm = module_id.module_graph_module(mg);
      // If the result is `None`, that means we have some differences with webpack,
      // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ModuleConcatenationPlugin.js#L168-L171
      if mg.is_async(&*module_id).expect("should have async result") {
        // TODO: bailout
        continue;
      }
      if !mgm
        .build_info
        .as_ref()
        .expect("should have module graph module")
        .strict
      {
        // TODO: bailout
        continue;
      }
      if chunk_graph.get_number_of_module_chunks(*module_id) == 0 {
        // TODO: bailout
        continue;
      }
      let exports_info = mg.get_exports_info(&*module_id);
      let relevnat_epxorts = exports_info.get_relevant_exports(None, mg);
      let unknown_exports = relevnat_epxorts
        .iter()
        .filter(|id| {
          let export_info = id.get_export_info_mut(mg).clone();
          // export_info.is_reexport() && export_info.get_target(mg, None).is_none()
          todo!()
        })
        .copied()
        .collect::<Vec<_>>();
      if unknown_exports.len() > 0 {
        // TODO: bailout
        continue;
      }
      let unknown_provided_exports = relevnat_epxorts
        .iter()
        .filter(|id| {
          let export_info = id.get_export_info(mg);
          !matches!(export_info.provided, Some(ExportInfoProvided::True))
        })
        .copied()
        .collect::<Vec<_>>();

      if unknown_provided_exports.len() > 0 {
        // TODO: bailout
        can_be_root = false;
      }

      if chunk_graph.is_entry_module(&*module_id) {
        // TODO: bailout
        can_be_inner = false;
      }
      if can_be_root {
        relevant_modules.push(*module_id);
      }
      if can_be_inner {
        possible_inners.insert(*module_id);
      }
    }

    logger.time_end(start);
    logger.debug(format!(
      "{} potential root modules, {} potential inner modules",
      relevant_modules.len(),
      possible_inners.len(),
    ));

    let start = logger.time("sort relevant modules");
    // relevant_modules.sort_by(|a, b| {
    //
    //
    //     });
    logger.time_end(start);
    Ok(())
  }
}

fn extends_harmony_dep(dep: &BoxDependency) -> bool {
  dep.is::<HarmonyExportImportedSpecifierDependency>()
    || dep.is::<HarmonyImportSideEffectDependency>()
    || dep.is::<HarmonyImportSpecifierDependency>()
}
