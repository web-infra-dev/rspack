use std::collections::hash_map::Entry;
use std::collections::VecDeque;

use rspack_core::{
  is_exports_object_referenced, is_no_exports_referenced, merge_runtime,
  AsyncDependenciesBlockIdentifier, BuildMetaExportsType, Compilation, ConnectionState,
  DependenciesBlock, DependencyId, ExportsInfoId, ExtendedReferencedExport, GroupOptions,
  ModuleIdentifier, Plugin, ReferencedExport, RuntimeSpec, UsageState,
};
use rspack_error::Result;
use rspack_identifier::IdentifierMap;
use rspack_util::swc::join_atom;
use rustc_hash::FxHashMap as HashMap;

#[derive(Debug)]
enum ModuleOrAsyncDependenciesBlock {
  Module(ModuleIdentifier),
  AsyncDependenciesBlock(AsyncDependenciesBlockIdentifier),
}
#[allow(unused)]
pub struct FlagDependencyUsagePluginProxy<'a> {
  global: bool,
  compilation: &'a mut Compilation,
  exports_info_module_map: HashMap<ExportsInfoId, ModuleIdentifier>,
}

#[allow(unused)]
impl<'a> FlagDependencyUsagePluginProxy<'a> {
  pub fn new(global: bool, compilation: &'a mut Compilation) -> Self {
    Self {
      global,
      compilation,
      exports_info_module_map: HashMap::default(),
    }
  }

  fn apply(&mut self) {
    for mgm in self
      .compilation
      .get_module_graph()
      .module_graph_modules()
      .values()
    {
      self
        .exports_info_module_map
        .insert(mgm.exports, mgm.module_identifier);
    }
    let mut q = VecDeque::new();
    let mg = self.compilation.get_module_graph_mut();
    // debug_exports_info!(mg);
    for exports_info_id in self.exports_info_module_map.keys() {
      exports_info_id.set_has_use_info(mg);
    }
    // SAFETY: we can make sure that entries will not be used other place at the same time,
    // this take is aiming to avoid use self ref and mut ref at the same time;
    let mut global_runtime: Option<RuntimeSpec> = None;
    let entries = std::mem::take(&mut self.compilation.entries);
    for (entry_name, entry) in entries.iter() {
      let runtime = if self.global {
        None
      } else {
        Some(
          self
            .compilation
            .get_entry_runtime(entry_name, Some(&entry.options)),
        )
      };
      if let Some(runtime) = runtime.as_ref() {
        let tem_global_runtime = global_runtime.get_or_insert_default();
        global_runtime = Some(merge_runtime(tem_global_runtime, runtime));
      }
      for &dep in entry.dependencies.iter() {
        self.process_entry_dependency(dep, runtime.clone(), &mut q);
      }
      for &dep in entry.include_dependencies.iter() {
        self.process_entry_dependency(dep, runtime.clone(), &mut q);
      }
    }
    for dep in self.compilation.global_entry.dependencies.clone() {
      self.process_entry_dependency(dep, global_runtime.clone(), &mut q);
    }
    for dep in self.compilation.global_entry.include_dependencies.clone() {
      self.process_entry_dependency(dep, global_runtime.clone(), &mut q);
    }
    self.compilation.entries = entries;

    while let Some((module_id, runtime)) = q.pop_front() {
      self.process_module(
        ModuleOrAsyncDependenciesBlock::Module(module_id),
        runtime,
        false,
        &mut q,
      );
    }
  }

  fn process_module(
    &mut self,
    block_id: ModuleOrAsyncDependenciesBlock,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
    q: &mut VecDeque<(ModuleIdentifier, Option<RuntimeSpec>)>,
  ) {
    #[derive(Debug, Clone)]
    enum ProcessModuleReferencedExports {
      Map(HashMap<String, ExtendedReferencedExport>),
      ExtendRef(Vec<ExtendedReferencedExport>),
    }

    let mut map: IdentifierMap<ProcessModuleReferencedExports> = IdentifierMap::default();
    let mut queue = VecDeque::new();
    queue.push_back(block_id);
    while let Some(module_id) = queue.pop_front() {
      // dbg!(&module_id);
      let (blocks, dependencies) = match module_id {
        ModuleOrAsyncDependenciesBlock::Module(module) => {
          let block = self
            .compilation
            .get_module_graph()
            .module_by_identifier(&module)
            .expect("should have module");
          (block.get_blocks(), block.get_dependencies())
        }
        ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(async_dependencies_block_id) => {
          let block = self
            .compilation
            .get_module_graph()
            .block_by_id(&async_dependencies_block_id)
            .expect("should have module");
          (block.get_blocks(), block.get_dependencies())
        }
      };
      let (block_id_list, dep_id_list) = (blocks.to_vec(), dependencies.to_vec());
      for block_id in block_id_list {
        let block = self
          .compilation
          .get_module_graph()
          .block_by_id(&block_id)
          .expect("should have block");
        if !self.global
          && let Some(GroupOptions::Entrypoint(options)) = block.get_group_options()
        {
          let runtime = options
            .runtime
            .as_ref()
            .map(|runtime| RuntimeSpec::from_iter([runtime.as_str().into()]));
          self.process_module(
            ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(block_id),
            runtime,
            true,
            q,
          )
        } else {
          queue.push_back(ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(
            block_id,
          ));
        }
      }
      for dep_id in dep_id_list.into_iter() {
        let connection = self
          .compilation
          .get_module_graph()
          .connection_by_dependency(&dep_id);

        let connection = if let Some(connection) = connection {
          connection
        } else {
          continue;
        };
        let active_state =
          connection.get_active_state(self.compilation.get_module_graph(), runtime.as_ref());

        // dbg!(
        //   &connection,
        //   dep_id
        //     .get_dependency(&self.compilation.get_module_graph())
        //     .dependency_debug_name(),
        //   active_state
        // );
        match active_state {
          ConnectionState::Bool(false) => {
            continue;
          }
          ConnectionState::TransitiveOnly => {
            self.process_module(
              ModuleOrAsyncDependenciesBlock::Module(connection.module_identifier),
              runtime.clone(),
              false,
              q,
            );
            continue;
          }
          _ => {}
        }
        let old_referenced_exports = map.remove(&connection.module_identifier);
        let dep = self
          .compilation
          .get_module_graph()
          .dependency_by_id(&dep_id)
          .expect("should have dep");

        let referenced_exports = if let Some(md) = dep.as_module_dependency() {
          md.get_referenced_exports(self.compilation.get_module_graph(), runtime.as_ref())
        } else if dep.as_context_dependency().is_some() {
          vec![ExtendedReferencedExport::Array(vec![])]
        } else {
          continue;
        };
        // dbg!(
        //   &connection,
        //   dep.dependency_debug_name(),
        //   &referenced_exports,
        //   &old_referenced_exports
        // );

        if old_referenced_exports.is_none()
          || matches!(old_referenced_exports, Some(ProcessModuleReferencedExports::ExtendRef(ref v)) if is_no_exports_referenced(v))
          || is_exports_object_referenced(&referenced_exports)
        {
          map.insert(
            connection.module_identifier,
            ProcessModuleReferencedExports::ExtendRef(referenced_exports),
          );
        } else if let Some(old_referenced_exports) = old_referenced_exports {
          if is_no_exports_referenced(&referenced_exports) {
            map.insert(connection.module_identifier, old_referenced_exports.clone());
            continue;
          }

          let mut exports_map = match old_referenced_exports {
            ProcessModuleReferencedExports::Map(map) => map,
            ProcessModuleReferencedExports::ExtendRef(ref_items) => {
              let mut exports_map = HashMap::default();
              for item in ref_items.iter() {
                match item {
                  ExtendedReferencedExport::Array(arr) => {
                    exports_map.insert(join_atom(arr.iter(), "\n"), item.clone());
                  }
                  ExtendedReferencedExport::Export(export) => {
                    exports_map.insert(join_atom(export.name.iter(), "\n"), item.clone());
                  }
                }
              }
              exports_map
            }
          };

          for mut item in referenced_exports.into_iter() {
            match item {
              ExtendedReferencedExport::Array(ref arr) => {
                let key = join_atom(arr.iter(), "\n");
                exports_map.entry(key).or_insert(item);
              }
              ExtendedReferencedExport::Export(ref mut export) => {
                let key = join_atom(export.name.iter(), "\n");
                match exports_map.entry(key) {
                  Entry::Occupied(mut occ) => {
                    let old_item = occ.get();
                    match old_item {
                      ExtendedReferencedExport::Array(_) => {
                        occ.insert(item);
                      }
                      ExtendedReferencedExport::Export(old_item) => {
                        occ.insert(ExtendedReferencedExport::Export(ReferencedExport {
                          name: std::mem::take(&mut export.name),
                          can_mangle: export.can_mangle && old_item.can_mangle,
                        }));
                      }
                    }
                  }
                  Entry::Vacant(vac) => {
                    vac.insert(item);
                  }
                }
              }
            }
          }
          // dbg!(&exports_map);
          map.insert(
            connection.module_identifier,
            ProcessModuleReferencedExports::Map(exports_map),
          );
        }
      }
    }

    for (module_id, referenced_exports) in map {
      // dbg!(&module_id, &referenced_exports);
      let normalized_refs = match referenced_exports {
        ProcessModuleReferencedExports::Map(map) => map.into_values().collect::<Vec<_>>(),
        ProcessModuleReferencedExports::ExtendRef(extend_ref) => extend_ref,
      };
      self.process_referenced_module(
        module_id,
        normalized_refs,
        runtime.clone(),
        force_side_effects,
        q,
      );
    }
  }

  fn process_entry_dependency(
    &mut self,
    dep: DependencyId,
    runtime: Option<RuntimeSpec>,
    queue: &mut VecDeque<(ModuleIdentifier, Option<RuntimeSpec>)>,
  ) {
    if let Some(module) = self
      .compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(&dep)
    {
      self.process_referenced_module(module.module_identifier, vec![], runtime, true, queue);
    }
  }

  fn process_referenced_module(
    &mut self,
    module_id: ModuleIdentifier,
    used_exports: Vec<ExtendedReferencedExport>,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
    queue: &mut VecDeque<(ModuleIdentifier, Option<RuntimeSpec>)>,
  ) {
    let mgm = self
      .compilation
      .get_module_graph()
      .module_graph_module_by_identifier(&module_id)
      .expect("should have mgm");
    let module = self
      .compilation
      .get_module_graph()
      .module_by_identifier(&module_id)
      .expect("should have module");
    let mgm_exports_info_id = mgm.exports;
    if !used_exports.is_empty() {
      let need_insert = match module.build_meta() {
        Some(build_meta) => matches!(build_meta.exports_type, BuildMetaExportsType::Unset),
        None => true,
      };
      if need_insert {
        let flag = mgm_exports_info_id
          .set_used_without_info(self.compilation.get_module_graph_mut(), runtime.as_ref());
        if flag {
          queue.push_back((module_id, None));
        }
        return;
      }

      for used_export_info in used_exports {
        let (can_mangle, used_exports) = match used_export_info {
          ExtendedReferencedExport::Array(used_exports) => (true, used_exports),
          ExtendedReferencedExport::Export(export) => (export.can_mangle, export.name),
        };
        if used_exports.is_empty() {
          let flag = mgm_exports_info_id
            .set_used_in_unknown_way(self.compilation.get_module_graph_mut(), runtime.as_ref());

          if flag {
            queue.push_back((module_id, runtime.clone()));
          }
        } else {
          let mut current_exports_info_id = mgm_exports_info_id;
          // dbg!(&current_exports_info_id.get_exports_info(&self.compilation.get_module_graph()));
          let len = used_exports.len();
          for (i, used_export) in used_exports.into_iter().enumerate() {
            let export_info_id = current_exports_info_id
              .get_export_info(&used_export, self.compilation.get_module_graph_mut());
            // dbg!(&export_info_id.get_export_info(&self.compilation.get_module_graph()));
            let export_info = self
              .compilation
              .get_module_graph_mut()
              .get_export_info_mut_by_id(&export_info_id);
            if !can_mangle {
              export_info.can_mangle_use = Some(false);
            }
            let last_one = i == len - 1;
            if !last_one {
              let nested_info =
                export_info_id.get_nested_exports_info(self.compilation.get_module_graph());
              // dbg!(&nested_info);
              if let Some(nested_info) = nested_info {
                let changed_flag = export_info_id.set_used_conditionally(
                  self.compilation.get_module_graph_mut(),
                  Box::new(|used| used == &UsageState::Unused),
                  UsageState::OnlyPropertiesUsed,
                  runtime.as_ref(),
                );
                if changed_flag {
                  let current_module = if current_exports_info_id == mgm_exports_info_id {
                    Some(module_id)
                  } else {
                    self
                      .exports_info_module_map
                      .get(&current_exports_info_id)
                      .cloned()
                  };
                  if let Some(current_module) = current_module {
                    queue.push_back((current_module, runtime.clone()));
                  }
                }
                current_exports_info_id = nested_info;
                continue;
              }
            }

            let changed_flag = export_info_id.set_used_conditionally(
              self.compilation.get_module_graph_mut(),
              Box::new(|v| v != &UsageState::Used),
              UsageState::Used,
              runtime.as_ref(),
            );
            // dbg!(
            //   &export_info_id.get_export_info(&self.compilation.get_module_graph()),
            //   changed_flag
            // );
            if changed_flag {
              let current_module = if current_exports_info_id == mgm_exports_info_id {
                Some(module_id)
              } else {
                self
                  .exports_info_module_map
                  .get(&current_exports_info_id)
                  .cloned()
              };
              if let Some(current_module) = current_module {
                queue.push_back((current_module, runtime.clone()));
              }
            }
            break;
          }
        }
      }
    } else {
      if !force_side_effects
        && match mgm.factory_meta {
          Some(ref meta) => meta.side_effect_free.unwrap_or_default(),
          None => false,
        }
      {
        return;
      }
      let changed_flag = mgm_exports_info_id
        .set_used_for_side_effects_only(self.compilation.get_module_graph_mut(), runtime.as_ref());
      if changed_flag {
        queue.push_back((module_id, runtime));
      }
    }
  }
}

#[derive(Debug)]
pub struct FlagDependencyUsagePlugin {
  global: bool,
}

impl FlagDependencyUsagePlugin {
  pub fn new(global: bool) -> Self {
    Self { global }
  }
}

#[async_trait::async_trait]
impl Plugin for FlagDependencyUsagePlugin {
  async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<()>> {
    let mut proxy = FlagDependencyUsagePluginProxy::new(self.global, compilation);
    proxy.apply();
    Ok(None)
  }
}
