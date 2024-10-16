use std::collections::hash_map::Entry;

use indexmap::IndexMap;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  ApplyContext, BuildMetaExportsType, Compilation, CompilationFinishModules, CompilerOptions,
  DependenciesBlock, DependencyId, ExportInfoProvided, ExportNameOrSpec, ExportsInfo,
  ExportsOfExportsSpec, ExportsSpec, ModuleGraph, ModuleGraphConnection, ModuleIdentifier, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::queue::Queue;
use swc_core::ecma::atoms::Atom;

struct FlagDependencyExportsState<'a> {
  mg: &'a mut ModuleGraph<'a>,
  changed: bool,
  current_module_id: ModuleIdentifier,
  dependencies: IdentifierMap<IdentifierSet>,
}

impl<'a> FlagDependencyExportsState<'a> {
  pub fn new(mg: &'a mut ModuleGraph<'a>) -> Self {
    Self {
      mg,
      changed: false,
      current_module_id: ModuleIdentifier::default(),
      dependencies: IdentifierMap::default(),
    }
  }

  pub fn apply(&mut self, modules: IdentifierSet) {
    let mut q = Queue::new();

    for module_id in modules {
      let mgm = self
        .mg
        .module_graph_module_by_identifier(&module_id)
        .expect("mgm should exist");
      let exports_info = mgm.exports;

      let module = self
        .mg
        .module_by_identifier(&mgm.module_identifier)
        .expect("should have module");
      let is_module_without_exports = if let Some(build_meta) = module.build_meta() {
        build_meta.exports_type == BuildMetaExportsType::Unset
      } else {
        true
      };
      if is_module_without_exports {
        let other_exports_info = exports_info.other_exports_info(self.mg);
        if !matches!(
          other_exports_info.provided(self.mg),
          Some(ExportInfoProvided::Null)
        ) {
          exports_info.set_has_provide_info(self.mg);
          exports_info.set_unknown_exports_provided(self.mg, false, None, None, None, None);
          continue;
        }
      }

      if !module
        .build_info()
        .as_ref()
        .map(|item| item.hash.is_some())
        .unwrap_or_default()
      {
        exports_info.set_has_provide_info(self.mg);
        q.enqueue(module_id);
        continue;
      }

      exports_info.set_has_provide_info(self.mg);
      q.enqueue(module_id);
    }

    while let Some(module_id) = q.dequeue() {
      self.changed = false;
      self.current_module_id = module_id;
      let mut exports_specs_from_dependencies: IndexMap<DependencyId, ExportsSpec> =
        IndexMap::default();
      self.process_dependencies_block(&module_id, &mut exports_specs_from_dependencies);
      let exports_info = self.mg.get_exports_info(&module_id);
      for (dep_id, exports_spec) in exports_specs_from_dependencies.into_iter() {
        self.process_exports_spec(dep_id, exports_spec, exports_info);
      }
      if self.changed {
        self.notify_dependencies(&mut q);
      }
    }
  }

  // #[tracing::instrument(skip_all, fields(module = ?self.current_module_id))]
  pub fn notify_dependencies(&mut self, q: &mut Queue<ModuleIdentifier>) {
    if let Some(set) = self.dependencies.get(&self.current_module_id) {
      for mi in set.iter() {
        q.enqueue(*mi);
      }
    }
  }

  pub fn process_dependencies_block(
    &self,
    module_identifier: &ModuleIdentifier,
    exports_specs_from_dependencies: &mut IndexMap<DependencyId, ExportsSpec>,
  ) -> Option<()> {
    let block = &**self.mg.module_by_identifier(module_identifier)?;
    self.process_dependencies_block_inner(block, exports_specs_from_dependencies)
  }

  fn process_dependencies_block_inner<B: DependenciesBlock + ?Sized>(
    &self,
    block: &B,
    exports_specs_from_dependencies: &mut IndexMap<DependencyId, ExportsSpec>,
  ) -> Option<()> {
    for dep_id in block.get_dependencies().iter() {
      let dep = self
        .mg
        .dependency_by_id(dep_id)
        .expect("should have dependency");
      self.process_dependency(
        *dep_id,
        dep.get_exports(self.mg),
        exports_specs_from_dependencies,
      );
    }
    for block_id in block.get_blocks() {
      let block = self.mg.block_by_id(block_id)?;
      self.process_dependencies_block_inner(block, exports_specs_from_dependencies);
    }
    None
  }

  pub fn process_dependency(
    &self,
    dep_id: DependencyId,
    exports_specs: Option<ExportsSpec>,
    exports_specs_from_dependencies: &mut IndexMap<DependencyId, ExportsSpec>,
  ) -> Option<()> {
    // this is why we can bubble here. https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/FlagDependencyExportsPlugin.js#L140
    let exports_specs = exports_specs?;
    exports_specs_from_dependencies.insert(dep_id, exports_specs);
    Some(())
  }

  pub fn process_exports_spec(
    &mut self,
    dep_id: DependencyId,
    export_desc: ExportsSpec,
    exports_info: ExportsInfo,
  ) {
    let exports = &export_desc.exports;
    // dbg!(&exports);
    let global_can_mangle = &export_desc.can_mangle;
    let global_from = export_desc.from.as_ref();
    let global_priority = &export_desc.priority;
    let global_terminal_binding = export_desc.terminal_binding.unwrap_or(false);
    let export_dependencies = &export_desc.dependencies;
    if let Some(hide_export) = export_desc.hide_export {
      for name in hide_export.iter() {
        let from_exports_info = exports_info.get_export_info(self.mg, name);
        from_exports_info.unset_target(self.mg, &dep_id);
      }
    }
    match exports {
      ExportsOfExportsSpec::True => {
        if exports_info.set_unknown_exports_provided(
          self.mg,
          global_can_mangle.unwrap_or_default(),
          export_desc.exclude_exports,
          global_from.map(|_| dep_id),
          global_from.map(|_| dep_id),
          *global_priority,
        ) {
          self.changed = true;
        };
      }
      ExportsOfExportsSpec::Null => {}
      ExportsOfExportsSpec::Array(ele) => {
        self.merge_exports(
          exports_info,
          ele,
          DefaultExportInfo {
            can_mangle: *global_can_mangle,
            terminal_binding: global_terminal_binding,
            from: global_from,
            priority: *global_priority,
          },
          dep_id,
        );
      }
    }

    if let Some(export_dependencies) = export_dependencies {
      for export_dep in export_dependencies {
        match self.dependencies.entry(*export_dep) {
          Entry::Occupied(mut occ) => {
            occ.get_mut().insert(self.current_module_id);
          }
          Entry::Vacant(vac) => {
            vac.insert(IdentifierSet::from_iter([self.current_module_id]));
          }
        }
      }
    }
  }

  pub fn merge_exports(
    &mut self,
    exports_info: ExportsInfo,
    exports: &Vec<ExportNameOrSpec>,
    global_export_info: DefaultExportInfo,
    dep_id: DependencyId,
  ) {
    for export_name_or_spec in exports {
      let (name, can_mangle, terminal_binding, exports, from, from_export, priority, hidden) =
        match export_name_or_spec {
          ExportNameOrSpec::String(name) => (
            name.clone(),
            global_export_info.can_mangle,
            global_export_info.terminal_binding,
            None::<&Vec<ExportNameOrSpec>>,
            global_export_info.from.cloned(),
            None::<&rspack_core::Nullable<Vec<Atom>>>,
            global_export_info.priority,
            false,
          ),
          ExportNameOrSpec::ExportSpec(spec) => (
            spec.name.clone(),
            match spec.can_mangle {
              Some(v) => Some(v),
              None => global_export_info.can_mangle,
            },
            spec
              .terminal_binding
              .unwrap_or(global_export_info.terminal_binding),
            spec.exports.as_ref(),
            if spec.from.is_some() {
              spec.from.clone()
            } else {
              global_export_info.from.cloned()
            },
            spec.export.as_ref(),
            match spec.priority {
              Some(v) => Some(v),
              None => global_export_info.priority,
            },
            spec.hidden.unwrap_or(false),
          ),
        };
      let export_info = exports_info.get_export_info(self.mg, &name);
      if let Some(provided) = export_info.provided(self.mg)
        && matches!(
          provided,
          ExportInfoProvided::False | ExportInfoProvided::Null
        )
      {
        export_info.set_provided(self.mg, Some(ExportInfoProvided::True));
        self.changed = true;
      }

      if Some(false) != export_info.can_mangle_provide(self.mg) && can_mangle == Some(false) {
        export_info.set_can_mangle_provide(self.mg, Some(false));
        self.changed = true;
      }

      if terminal_binding && !export_info.terminal_binding(self.mg) {
        export_info.set_terminal_binding(self.mg, true);
        self.changed = true;
      }

      if let Some(exports) = exports {
        let nested_exports_info = export_info.create_nested_exports_info(self.mg);
        self.merge_exports(
          nested_exports_info,
          exports,
          global_export_info.clone(),
          dep_id,
        );
      }

      // shadowing the previous `export_info_mut` to reduce the mut borrow life time,
      // because `create_nested_exports_info` needs `&mut ModuleGraph`
      if let Some(from) = from {
        let changed = if hidden {
          export_info.unset_target(self.mg, &dep_id)
        } else {
          let fallback = rspack_core::Nullable::Value(vec![name.clone()]);
          let export_name = if let Some(from) = from_export {
            Some(from)
          } else {
            Some(&fallback)
          };
          export_info.set_target(
            self.mg,
            Some(dep_id),
            Some(from.dependency_id),
            export_name,
            priority,
          )
        };
        self.changed |= changed;
      }

      // Recalculate target exportsInfo
      let target = export_info.get_target(self.mg);

      let mut target_exports_info: Option<ExportsInfo> = None;
      if let Some(target) = target {
        let target_module_exports_info = self.mg.get_exports_info(&target.module);
        target_exports_info =
          target_module_exports_info.get_nested_exports_info(self.mg, target.export);
        match self.dependencies.entry(target.module) {
          Entry::Occupied(mut occ) => {
            occ.get_mut().insert(self.current_module_id);
          }
          Entry::Vacant(vac) => {
            vac.insert(IdentifierSet::from_iter([self.current_module_id]));
          }
        }
      }

      if export_info.exports_info_owned(self.mg) {
        let changed = export_info
          .exports_info(self.mg)
          .expect("should have exports_info when exports_info_owned is true")
          .set_redirect_name_to(self.mg, target_exports_info);
        if changed {
          self.changed = true;
        }
      } else if export_info.exports_info(self.mg) != target_exports_info {
        export_info.set_exports_info(self.mg, target_exports_info);
        self.changed = true;
      }
    }
  }
}

/// Used for reducing nums of params
#[derive(Debug, Clone)]
pub struct DefaultExportInfo<'a> {
  can_mangle: Option<bool>,
  terminal_binding: bool,
  from: Option<&'a ModuleGraphConnection>,
  priority: Option<u8>,
}

#[plugin]
#[derive(Debug, Default)]
pub struct FlagDependencyExportsPlugin;

#[plugin_hook(CompilationFinishModules for FlagDependencyExportsPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let modules: IdentifierSet = if compilation.options.incremental().provided_exports_enabled() {
    compilation
      .unaffected_modules_cache
      .get_affected_modules_with_module_graph()
      .lock()
      .expect("should lock")
      .clone()
  } else {
    compilation
      .get_module_graph()
      .modules()
      .keys()
      .copied()
      .collect()
  };
  FlagDependencyExportsState::new(&mut compilation.get_module_graph_mut()).apply(modules);
  Ok(())
}

impl Plugin for FlagDependencyExportsPlugin {
  fn name(&self) -> &'static str {
    "FlagDependencyExportsPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}
