use std::{
  fs,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use rspack_core::{
  AssetInfo, Compilation, CompilationParams, CompilationProcessAssets, CompilerCompilation,
  CompilerMake, Dependency, DependencyId, DependencyType, EntryDependency, EntryOptions,
  ModuleIdentifier, ModuleType, Plugin,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::{Result, error};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::parser_and_generator::{
  DTS_MODULE_TYPE, DtsDeclKey, DtsModuleData, DtsModuleStore, DtsParserAndGenerator,
  apply_name_mapping, parse_dts_source, render_module_item, strip_export_from_item, to_dts_request,
};

#[derive(Debug, Clone)]
pub struct DtsPluginEntry {
  pub name: String,
  pub request: String,
}

#[derive(Debug, Clone)]
pub struct DtsPluginOptions {
  pub entries: Vec<DtsPluginEntry>,
  pub filename: String,
  pub externals: Vec<String>,
}

#[derive(Debug, Clone)]
struct CompilationEntryRef {
  name: String,
  request: String,
  dependency_id: DependencyId,
}

#[derive(Debug, Default)]
struct DtsPluginState {
  entries_by_compilation: FxHashMap<rspack_core::CompilationId, Vec<CompilationEntryRef>>,
}

#[plugin]
#[derive(Debug)]
pub struct DtsPlugin {
  options: DtsPluginOptions,
  store: DtsModuleStore,
  state: Arc<Mutex<DtsPluginState>>,
  external_requests: Arc<FxHashSet<String>>,
}

impl DtsPlugin {
  pub fn new(options: DtsPluginOptions) -> Self {
    let external_requests = Arc::new(options.externals.iter().cloned().collect());
    Self::new_inner(
      options,
      Arc::new(Mutex::new(Default::default())),
      Arc::new(Mutex::new(Default::default())),
      external_requests,
    )
  }

  fn render_filename(&self, name: &str) -> String {
    self.options.filename.replace("[name]", name)
  }
}

#[plugin_hook(CompilerCompilation for DtsPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
  Ok(())
}

#[plugin_hook(CompilerMake for DtsPlugin, stage = 1000)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let mut entry_refs = Vec::with_capacity(self.options.entries.len());
  for entry in &self.options.entries {
    let dependency = EntryDependency::new(
      to_dts_request(&entry.request),
      compilation.options.context.clone(),
      None,
      true,
    );
    let dependency_id = *dependency.id();
    compilation
      .add_entry(
        Box::new(dependency),
        EntryOptions {
          name: Some(format!("__rspack_dts__{}", entry.name)),
          ..Default::default()
        },
      )
      .await?;
    entry_refs.push(CompilationEntryRef {
      name: entry.name.clone(),
      request: entry.request.clone(),
      dependency_id,
    });
  }

  self
    .state
    .lock()
    .expect("dts plugin state lock poisoned")
    .entries_by_compilation
    .insert(compilation.id(), entry_refs);
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for DtsPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let entry_refs = self
    .state
    .lock()
    .expect("dts plugin state lock poisoned")
    .entries_by_compilation
    .get(&compilation.id())
    .cloned()
    .unwrap_or_default();
  if entry_refs.is_empty() {
    return Ok(());
  }

  for entry in entry_refs {
    let code = self.bundle_entry(compilation, &entry)?;
    compilation.emit_asset(
      self.render_filename(&entry.name),
      rspack_core::CompilationAsset::new(
        Some(RawStringSource::from(code).boxed()),
        AssetInfo::default(),
      ),
    );
  }

  let hidden_assets = compilation
    .assets()
    .keys()
    .filter(|filename| filename.starts_with("__rspack_dts__"))
    .cloned()
    .collect::<Vec<_>>();
  for asset in hidden_assets {
    compilation.assets_mut().remove(&asset);
  }

  Ok(())
}

impl DtsPlugin {
  fn bundle_entry(&self, compilation: &Compilation, entry: &CompilationEntryRef) -> Result<String> {
    let modules = self.store.lock().expect("dts module store lock poisoned");
    if modules.is_empty() {
      drop(modules);
      return self.bundle_entry_direct(compilation, entry);
    }
    let module_graph = compilation.get_module_graph();
    let root_identifier = module_graph
      .module_identifier_by_dependency_id(&entry.dependency_id)
      .copied()
      .or_else(|| {
        modules
          .keys()
          .find(|identifier| identifier.to_string().contains(&entry.request))
          .copied()
      })
      .ok_or_else(|| {
        error!(
          "Failed to resolve dts entry module for {} (request: {}). Known dts modules: {}",
          entry.name,
          entry.request,
          modules
            .keys()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
        )
      })?;

    let root_data = modules
      .get(&root_identifier)
      .ok_or_else(|| error!("Missing dts summary for root module {}", root_identifier))?;

    let mut export_bindings: FxHashMap<String, DtsDeclKey> = FxHashMap::default();
    let mut external_reexports = vec![];
    let mut resolving = FxHashSet::default();
    resolve_module_exports(
      compilation,
      &modules,
      &root_data.summary.module_identifier,
      &mut export_bindings,
      &mut external_reexports,
      &mut resolving,
    )?;

    let mut kept = FxHashSet::<String>::default();
    let mut queued = export_bindings.values().cloned().collect::<Vec<_>>();
    while let Some(key) = queued.pop() {
      let key_id = key.id();
      if !kept.insert(key_id) {
        continue;
      }
      let module_identifier = ModuleIdentifier::from(key.module_identifier.as_str());
      let data = modules
        .get(&module_identifier)
        .ok_or_else(|| error!("Missing dts module {} while linking", key.module_identifier))?;
      let decl = data
        .summary
        .declarations
        .get(&key.local_name)
        .ok_or_else(|| {
          error!(
            "Missing declaration {} in {}",
            key.local_name, key.module_identifier
          )
        })?;
      for reference in &decl.references {
        if let Some(local) = data.summary.declarations.get(reference) {
          queued.push(local.key.clone());
          continue;
        }
        if let Some(binding) = data.summary.imports.get(reference)
          && !binding.is_external
          && let Some(dep_id) = binding.dependency_id
          && let Some(target_identifier) = module_graph.module_identifier_by_dependency_id(&dep_id)
        {
          let target_module = modules
            .get(target_identifier)
            .ok_or_else(|| error!("Missing imported dts module {}", target_identifier))?;
          let mut target_exports = FxHashMap::default();
          let mut noop_external = vec![];
          let mut noop_resolving = FxHashSet::default();
          resolve_module_exports(
            compilation,
            &modules,
            &target_module.summary.module_identifier,
            &mut target_exports,
            &mut noop_external,
            &mut noop_resolving,
          )?;
          if let Some(target_key) = target_exports.get(&binding.imported) {
            queued.push(target_key.clone());
          }
        }
      }
    }

    let mut kept_keys = kept
      .into_iter()
      .map(|id| parse_decl_key(&id))
      .collect::<Vec<_>>();
    kept_keys.sort_by(|a, b| a.id().cmp(&b.id()));

    let mut used_names = FxHashMap::<String, usize>::default();
    let mut final_names = FxHashMap::<String, String>::default();
    for key in &kept_keys {
      let current = used_names.entry(key.local_name.clone()).or_insert(0);
      let final_name = if *current == 0 {
        key.local_name.clone()
      } else {
        format!("{}${current}", key.local_name)
      };
      *current += 1;
      final_names.insert(key.id(), final_name);
    }

    let mut rendered = vec![];
    let mut emitted = FxHashSet::default();
    for key in &kept_keys {
      emit_decl(
        compilation,
        &modules,
        key,
        &final_names,
        &mut emitted,
        &mut rendered,
      )?;
    }

    let mut code = String::new();
    for item in external_reexports {
      code.push_str(&item);
      if !item.ends_with('\n') {
        code.push('\n');
      }
    }

    for decl in rendered {
      code.push_str(&decl.code);
      if !decl.code.ends_with('\n') {
        code.push('\n');
      }
    }

    if !export_bindings.is_empty() {
      let mut exports = export_bindings
        .into_iter()
        .map(|(exported, key)| {
          let final_name = final_names
            .get(&key.id())
            .cloned()
            .unwrap_or_else(|| key.local_name.clone());
          if exported == final_name {
            exported
          } else {
            format!("{final_name} as {exported}")
          }
        })
        .collect::<Vec<_>>();
      exports.sort();
      code.push_str("export { ");
      code.push_str(&exports.join(", "));
      code.push_str(" };\n");
    }

    Ok(code)
  }

  fn bundle_entry_direct(
    &self,
    compilation: &Compilation,
    entry: &CompilationEntryRef,
  ) -> Result<String> {
    let root_path = normalize_entry_path(compilation.options.context.as_ref(), &entry.request);
    let mut modules = FxHashMap::default();
    load_direct_module_recursive(&root_path, self.external_requests.as_ref(), &mut modules)?;

    let root_identifier = ModuleIdentifier::from(root_path.to_string_lossy().to_string());
    let mut export_bindings = FxHashMap::default();
    let mut external_reexports = vec![];
    let mut resolving = FxHashSet::default();
    resolve_module_exports_direct(
      &modules,
      &root_identifier,
      &mut export_bindings,
      &mut external_reexports,
      &mut resolving,
    )?;

    let mut kept = FxHashSet::<String>::default();
    let mut queued = export_bindings.values().cloned().collect::<Vec<_>>();
    while let Some(key) = queued.pop() {
      let key_id = key.id();
      if !kept.insert(key_id) {
        continue;
      }
      let module_identifier = ModuleIdentifier::from(key.module_identifier.as_str());
      let data = modules
        .get(&module_identifier)
        .ok_or_else(|| error!("Missing dts module {} while linking", key.module_identifier))?;
      let decl = data
        .summary
        .declarations
        .get(&key.local_name)
        .ok_or_else(|| error!("Missing declaration {}", key.local_name))?;
      for reference in &decl.references {
        if let Some(local) = data.summary.declarations.get(reference) {
          queued.push(local.key.clone());
          continue;
        }
        if let Some(binding) = data.summary.imports.get(reference)
          && !binding.is_external
        {
          let target_identifier = ModuleIdentifier::from(
            resolve_import_path(Path::new(&key.module_identifier), &binding.request)
              .to_string_lossy()
              .to_string(),
          );
          let target_module = modules
            .get(&target_identifier)
            .ok_or_else(|| error!("Missing imported dts module {}", target_identifier))?;
          let mut target_exports = FxHashMap::default();
          let mut noop_external = vec![];
          let mut noop_resolving = FxHashSet::default();
          resolve_module_exports_direct(
            &modules,
            &target_module.summary.module_identifier,
            &mut target_exports,
            &mut noop_external,
            &mut noop_resolving,
          )?;
          if let Some(target_key) = target_exports.get(&binding.imported) {
            queued.push(target_key.clone());
          }
        }
      }
    }

    let mut kept_keys = kept
      .into_iter()
      .map(|id| parse_decl_key(&id))
      .collect::<Vec<_>>();
    kept_keys.sort_by(|a, b| a.id().cmp(&b.id()));

    let mut used_names = FxHashMap::<String, usize>::default();
    let mut final_names = FxHashMap::<String, String>::default();
    for key in &kept_keys {
      let current = used_names.entry(key.local_name.clone()).or_insert(0);
      let final_name = if *current == 0 {
        key.local_name.clone()
      } else {
        format!("{}${current}", key.local_name)
      };
      *current += 1;
      final_names.insert(key.id(), final_name);
    }

    let mut rendered = vec![];
    let mut emitted = FxHashSet::default();
    for key in &kept_keys {
      emit_decl_direct(&modules, key, &final_names, &mut emitted, &mut rendered)?;
    }

    let mut code = String::new();
    for item in external_reexports {
      code.push_str(&item);
      if !item.ends_with('\n') {
        code.push('\n');
      }
    }
    for decl in rendered {
      code.push_str(&decl.code);
      if !decl.code.ends_with('\n') {
        code.push('\n');
      }
    }
    if !export_bindings.is_empty() {
      let mut exports = export_bindings
        .into_iter()
        .map(|(exported, key)| {
          let final_name = final_names
            .get(&key.id())
            .cloned()
            .unwrap_or_else(|| key.local_name.clone());
          if exported == final_name {
            exported
          } else {
            format!("{final_name} as {exported}")
          }
        })
        .collect::<Vec<_>>();
      exports.sort();
      code.push_str("export { ");
      code.push_str(&exports.join(", "));
      code.push_str(" };\n");
    }

    Ok(code)
  }
}

fn emit_decl(
  compilation: &Compilation,
  modules: &FxHashMap<ModuleIdentifier, Arc<DtsModuleData>>,
  key: &DtsDeclKey,
  final_names: &FxHashMap<String, String>,
  emitted: &mut FxHashSet<String>,
  rendered: &mut Vec<crate::parser_and_generator::DtsRenderDecl>,
) -> Result<()> {
  let key_id = key.id();
  if !emitted.insert(key_id.clone()) {
    return Ok(());
  }

  let module_identifier = ModuleIdentifier::from(key.module_identifier.as_str());
  let module = modules
    .get(&module_identifier)
    .ok_or_else(|| error!("Missing module {} while rendering", key.module_identifier))?;
  let decl = module
    .summary
    .declarations
    .get(&key.local_name)
    .ok_or_else(|| error!("Missing declaration {}", key.local_name))?;

  for reference in &decl.references {
    if let Some(local_decl) = module.summary.declarations.get(reference) {
      emit_decl(
        compilation,
        modules,
        &local_decl.key,
        final_names,
        emitted,
        rendered,
      )?;
      continue;
    }
    if let Some(import_binding) = module.summary.imports.get(reference)
      && !import_binding.is_external
      && let Some(dep_id) = import_binding.dependency_id
      && let Some(target_identifier) = compilation
        .get_module_graph()
        .module_identifier_by_dependency_id(&dep_id)
    {
      let target_module = modules
        .get(target_identifier)
        .ok_or_else(|| error!("Missing target module {}", target_identifier))?;
      let mut target_exports = FxHashMap::default();
      let mut noop_external = vec![];
      let mut resolving = FxHashSet::default();
      resolve_module_exports(
        compilation,
        modules,
        &target_module.summary.module_identifier,
        &mut target_exports,
        &mut noop_external,
        &mut resolving,
      )?;
      if let Some(target_key) = target_exports.get(&import_binding.imported) {
        emit_decl(
          compilation,
          modules,
          target_key,
          final_names,
          emitted,
          rendered,
        )?;
      }
    }
  }

  let mut item = strip_export_from_item(&decl.item);
  let mut mapping = FxHashMap::default();
  if let Some(final_name) = final_names.get(&key_id)
    && final_name != &key.local_name
  {
    mapping.insert(key.local_name.clone(), final_name.clone());
  }
  for reference in &decl.references {
    if let Some(local_decl) = module.summary.declarations.get(reference)
      && let Some(final_name) = final_names.get(&local_decl.key.id())
      && final_name != reference
    {
      mapping.insert(reference.clone(), final_name.clone());
    } else if let Some(import_binding) = module.summary.imports.get(reference)
      && !import_binding.is_external
      && let Some(dep_id) = import_binding.dependency_id
      && let Some(target_identifier) = compilation
        .get_module_graph()
        .module_identifier_by_dependency_id(&dep_id)
      && let Some(target_module) = modules.get(target_identifier)
    {
      let mut target_exports = FxHashMap::default();
      let mut noop_external = vec![];
      let mut resolving = FxHashSet::default();
      resolve_module_exports(
        compilation,
        modules,
        &target_module.summary.module_identifier,
        &mut target_exports,
        &mut noop_external,
        &mut resolving,
      )?;
      if let Some(target_key) = target_exports.get(&import_binding.imported)
        && let Some(final_name) = final_names.get(&target_key.id())
        && final_name != reference
      {
        mapping.insert(reference.clone(), final_name.clone());
      }
    }
  }
  apply_name_mapping(&mut item, &mapping);
  let code = render_module_item(&item)?;
  rendered.push(crate::parser_and_generator::DtsRenderDecl { code });
  Ok(())
}

fn resolve_module_exports(
  compilation: &Compilation,
  modules: &FxHashMap<ModuleIdentifier, Arc<DtsModuleData>>,
  module_identifier: &ModuleIdentifier,
  exports: &mut FxHashMap<String, DtsDeclKey>,
  external_reexports: &mut Vec<String>,
  resolving: &mut FxHashSet<ModuleIdentifier>,
) -> Result<()> {
  if !resolving.insert(*module_identifier) {
    return Ok(());
  }
  let module = modules
    .get(module_identifier)
    .ok_or_else(|| error!("Missing dts summary for {}", module_identifier))?;

  for decl in module.summary.declarations.values() {
    for export_name in &decl.export_names {
      exports.insert(export_name.clone(), decl.key.clone());
    }
  }

  for reexport in &module.summary.named_reexports {
    if reexport.is_external {
      external_reexports.push(render_module_item(&reexport.item)?);
      continue;
    }
    if let Some(dep_id) = reexport.dependency_id
      && let Some(target_identifier) = compilation
        .get_module_graph()
        .module_identifier_by_dependency_id(&dep_id)
    {
      let mut target_exports = FxHashMap::default();
      resolve_module_exports(
        compilation,
        modules,
        target_identifier,
        &mut target_exports,
        external_reexports,
        resolving,
      )?;
      if let Some(target_key) = target_exports.get(&reexport.imported) {
        exports.insert(reexport.exported.clone(), target_key.clone());
      }
    }
  }

  for reexport in &module.summary.star_reexports {
    if reexport.is_external {
      external_reexports.push(render_module_item(&reexport.item)?);
      continue;
    }
    if let Some(dep_id) = reexport.dependency_id
      && let Some(target_identifier) = compilation
        .get_module_graph()
        .module_identifier_by_dependency_id(&dep_id)
    {
      let mut target_exports = FxHashMap::default();
      resolve_module_exports(
        compilation,
        modules,
        target_identifier,
        &mut target_exports,
        external_reexports,
        resolving,
      )?;
      for (name, key) in target_exports {
        if name == "default" {
          continue;
        }
        exports.entry(name).or_insert(key);
      }
    }
  }

  resolving.remove(module_identifier);
  Ok(())
}

fn emit_decl_direct(
  modules: &FxHashMap<ModuleIdentifier, Arc<DtsModuleData>>,
  key: &DtsDeclKey,
  final_names: &FxHashMap<String, String>,
  emitted: &mut FxHashSet<String>,
  rendered: &mut Vec<crate::parser_and_generator::DtsRenderDecl>,
) -> Result<()> {
  let key_id = key.id();
  if !emitted.insert(key_id.clone()) {
    return Ok(());
  }

  let module_identifier = ModuleIdentifier::from(key.module_identifier.as_str());
  let module = modules
    .get(&module_identifier)
    .ok_or_else(|| error!("Missing module {} while rendering", key.module_identifier))?;
  let decl = module
    .summary
    .declarations
    .get(&key.local_name)
    .ok_or_else(|| error!("Missing declaration {}", key.local_name))?;

  for reference in &decl.references {
    if let Some(local_decl) = module.summary.declarations.get(reference) {
      emit_decl_direct(modules, &local_decl.key, final_names, emitted, rendered)?;
      continue;
    }
    if let Some(import_binding) = module.summary.imports.get(reference)
      && !import_binding.is_external
    {
      let target_identifier = ModuleIdentifier::from(
        resolve_import_path(Path::new(&key.module_identifier), &import_binding.request)
          .to_string_lossy()
          .to_string(),
      );
      let target_module = modules
        .get(&target_identifier)
        .ok_or_else(|| error!("Missing target module {}", target_identifier))?;
      let mut target_exports = FxHashMap::default();
      let mut noop_external = vec![];
      let mut resolving = FxHashSet::default();
      resolve_module_exports_direct(
        modules,
        &target_module.summary.module_identifier,
        &mut target_exports,
        &mut noop_external,
        &mut resolving,
      )?;
      if let Some(target_key) = target_exports.get(&import_binding.imported) {
        emit_decl_direct(modules, target_key, final_names, emitted, rendered)?;
      }
    }
  }

  let mut item = strip_export_from_item(&decl.item);
  let mut mapping = FxHashMap::default();
  if let Some(final_name) = final_names.get(&key_id)
    && final_name != &key.local_name
  {
    mapping.insert(key.local_name.clone(), final_name.clone());
  }
  for reference in &decl.references {
    if let Some(local_decl) = module.summary.declarations.get(reference)
      && let Some(final_name) = final_names.get(&local_decl.key.id())
      && final_name != reference
    {
      mapping.insert(reference.clone(), final_name.clone());
    } else if let Some(import_binding) = module.summary.imports.get(reference)
      && !import_binding.is_external
    {
      let target_identifier = ModuleIdentifier::from(
        resolve_import_path(Path::new(&key.module_identifier), &import_binding.request)
          .to_string_lossy()
          .to_string(),
      );
      if let Some(target_module) = modules.get(&target_identifier) {
        let mut target_exports = FxHashMap::default();
        let mut noop_external = vec![];
        let mut resolving = FxHashSet::default();
        resolve_module_exports_direct(
          modules,
          &target_module.summary.module_identifier,
          &mut target_exports,
          &mut noop_external,
          &mut resolving,
        )?;
        if let Some(target_key) = target_exports.get(&import_binding.imported)
          && let Some(final_name) = final_names.get(&target_key.id())
          && final_name != reference
        {
          mapping.insert(reference.clone(), final_name.clone());
        }
      }
    }
  }
  apply_name_mapping(&mut item, &mapping);
  let code = render_module_item(&item)?;
  rendered.push(crate::parser_and_generator::DtsRenderDecl { code });
  Ok(())
}

fn resolve_module_exports_direct(
  modules: &FxHashMap<ModuleIdentifier, Arc<DtsModuleData>>,
  module_identifier: &ModuleIdentifier,
  exports: &mut FxHashMap<String, DtsDeclKey>,
  external_reexports: &mut Vec<String>,
  resolving: &mut FxHashSet<ModuleIdentifier>,
) -> Result<()> {
  if !resolving.insert(*module_identifier) {
    return Ok(());
  }
  let module = modules
    .get(module_identifier)
    .ok_or_else(|| error!("Missing dts summary for {}", module_identifier))?;

  for decl in module.summary.declarations.values() {
    for export_name in &decl.export_names {
      exports.insert(export_name.clone(), decl.key.clone());
    }
  }

  for reexport in &module.summary.named_reexports {
    if reexport.is_external {
      external_reexports.push(render_module_item(&reexport.item)?);
      continue;
    }
    let target_identifier = ModuleIdentifier::from(
      resolve_import_path(Path::new(&module_identifier.to_string()), &reexport.request)
        .to_string_lossy()
        .to_string(),
    );
    let mut target_exports = FxHashMap::default();
    resolve_module_exports_direct(
      modules,
      &target_identifier,
      &mut target_exports,
      external_reexports,
      resolving,
    )?;
    if let Some(target_key) = target_exports.get(&reexport.imported) {
      exports.insert(reexport.exported.clone(), target_key.clone());
    }
  }

  for reexport in &module.summary.star_reexports {
    if reexport.is_external {
      external_reexports.push(render_module_item(&reexport.item)?);
      continue;
    }
    let target_identifier = ModuleIdentifier::from(
      resolve_import_path(Path::new(&module_identifier.to_string()), &reexport.request)
        .to_string_lossy()
        .to_string(),
    );
    let mut target_exports = FxHashMap::default();
    resolve_module_exports_direct(
      modules,
      &target_identifier,
      &mut target_exports,
      external_reexports,
      resolving,
    )?;
    for (name, key) in target_exports {
      if name == "default" {
        continue;
      }
      exports.entry(name).or_insert(key);
    }
  }

  resolving.remove(module_identifier);
  Ok(())
}

fn load_direct_module_recursive(
  path: &Path,
  external_requests: &FxHashSet<String>,
  modules: &mut FxHashMap<ModuleIdentifier, Arc<DtsModuleData>>,
) -> Result<()> {
  let identifier = ModuleIdentifier::from(path.to_string_lossy().to_string());
  if modules.contains_key(&identifier) {
    return Ok(());
  }
  let source = fs::read_to_string(path)
    .map_err(|err| error!("Failed to read dts module {}: {err}", path.display()))?;
  let summary = parse_dts_source(identifier, source, external_requests)?;
  modules.insert(
    identifier,
    Arc::new(DtsModuleData {
      summary: summary.clone(),
    }),
  );

  for binding in summary.imports.values() {
    if !binding.is_external {
      let target = resolve_import_path(path, &binding.request);
      load_direct_module_recursive(&target, external_requests, modules)?;
    }
  }
  for reexport in &summary.named_reexports {
    if !reexport.is_external {
      let target = resolve_import_path(path, &reexport.request);
      load_direct_module_recursive(&target, external_requests, modules)?;
    }
  }
  for reexport in &summary.star_reexports {
    if !reexport.is_external {
      let target = resolve_import_path(path, &reexport.request);
      load_direct_module_recursive(&target, external_requests, modules)?;
    }
  }
  Ok(())
}

fn resolve_import_path(current_path: &Path, request: &str) -> PathBuf {
  let base = current_path.parent().unwrap_or(current_path);
  let candidate = base.join(request);
  if candidate.exists() {
    return candidate;
  }
  let dts_candidate = PathBuf::from(format!("{}.d.ts", candidate.display()));
  if dts_candidate.exists() {
    return dts_candidate;
  }
  candidate
}

fn normalize_entry_path(context: &Path, request: &str) -> PathBuf {
  let candidate = context.join(request);
  if candidate.exists() {
    candidate
  } else {
    PathBuf::from(request)
  }
}

fn parse_decl_key(key: &str) -> DtsDeclKey {
  let parts = key.split("::").collect::<Vec<_>>();
  let module_identifier = parts.first().cloned().unwrap_or_default().to_string();
  let local_name = parts.get(1).cloned().unwrap_or_default().to_string();
  let space = match parts.get(2).copied().unwrap_or("Type") {
    "Value" => crate::parser_and_generator::DtsDeclSpace::Value,
    "Both" => crate::parser_and_generator::DtsDeclSpace::Both,
    _ => crate::parser_and_generator::DtsDeclSpace::Type,
  };
  DtsDeclKey {
    module_identifier,
    local_name,
    space,
  }
}

impl Plugin for DtsPlugin {
  fn name(&self) -> &'static str {
    "rspack.DtsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    let store = self.store.clone();
    let external_requests = self.external_requests.clone();
    ctx.register_parser_and_generator_builder(
      ModuleType::Custom(DTS_MODULE_TYPE.into()),
      Box::new(move |_parser, _generator| {
        Box::new(DtsParserAndGenerator::new(
          store.clone(),
          external_requests.clone(),
        ))
      }),
    );
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.make.tap(make::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }

  fn clear_cache(&self, id: rspack_core::CompilationId) {
    self
      .state
      .lock()
      .expect("dts plugin state lock poisoned")
      .entries_by_compilation
      .remove(&id);
  }
}
