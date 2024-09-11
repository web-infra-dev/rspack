use std::collections::{HashMap, HashSet};
use std::time::Instant;

use async_trait::async_trait;
use indexmap::set::IndexSet;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{
  ApplyContext, AssetInfo, Compilation, CompilationAsset, CompilationProcessAssets,
  CompilerFinishMake, CompilerOptions, EntryDependency, EntryOptions, Module, ModuleType, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde_json::to_string;

use crate::utils::decl::{ClientImports, ReactRoute};
use crate::utils::file::generate_asset_version;
use crate::utils::sever_reference::RSCServerReferenceManifest;
use crate::utils::shared_data::{SHARED_CLIENT_IMPORTS, SHARED_SERVER_IMPORTS};
use crate::utils::{has_client_directive, has_server_directive};

#[derive(Debug, Default, Clone)]
pub struct RSCClientEntryRspackPluginOptions {
  pub routes: Vec<ReactRoute>,
}

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct RSCClientEntryRspackPlugin {
  pub options: RSCClientEntryRspackPluginOptions,
}

static CSS_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.(css|scss|sass)").expect("css regexp init failed"));

impl RSCClientEntryRspackPlugin {
  pub fn new(options: RSCClientEntryRspackPluginOptions) -> Self {
    Self::new_inner(options)
  }
  async fn add_entry(&self, compilation: &mut Compilation) -> Result<()> {
    // TODO: multiple server entry support
    let context = compilation.options.context.clone();
    let request = format!(
      "rsc-server-action-entry-loader.js?from={}&name={}",
      "server-entry", "server-entry"
    );
    let entry = Box::new(EntryDependency::new(request, context.clone(), None, false));
    compilation
      .add_include(
        entry,
        EntryOptions {
          name: Some(String::from("server-entry")),
          ..Default::default()
        },
      )
      .await?;
    Ok(())
  }
  fn insert_client_imports(
    &self,
    client_imports: &mut HashMap<String, IndexSet<String>>,
    name: Option<&str>,
    client_import: &str,
  ) {
    if let Some(name) = name {
      let named_client_imports = client_imports
        .entry(name.into())
        .or_insert_with(IndexSet::default);
      named_client_imports.insert(client_import.into());
    }
  }
  fn get_route_entry(&self, resource: &str) -> Option<&ReactRoute> {
    self.options.routes.iter().find(|&f| f.import == resource)
  }
  fn is_visited(
    &self,
    visited_modules_by_entry: &mut HashMap<String, HashSet<String>>,
    entry: &str,
    route_entry: Option<&ReactRoute>,
    resource_path: &str,
  ) -> bool {
    let entry_key = route_entry.map_or_else(
      || entry.to_string(),
      |route_entry| route_entry.import.clone(),
    );
    let visited_modules = visited_modules_by_entry.get(&entry_key);
    visited_modules
      .map(|f| f.contains(resource_path))
      .unwrap_or(false)
  }
  fn mark_visited(
    &self,
    visited_modules_by_entry: &mut HashMap<String, HashSet<String>>,
    entry: &str,
    route_entry: Option<&ReactRoute>,
    resource_path: &str,
  ) {
    let entry_key = route_entry.map_or_else(
      || entry.to_string(),
      |route_entry| route_entry.import.clone(),
    );
    let visited_modules = visited_modules_by_entry
      .entry(entry_key)
      .or_insert_with(HashSet::new);
    visited_modules.insert(resource_path.into());
  }
  fn filter_client_components(
    &self,
    compilation: &Compilation,
    entry_name: &str,
    module: &Box<dyn Module>,
    visited_modules: &mut HashMap<String, HashSet<String>>,
    client_imports: &mut HashMap<String, IndexSet<String>>,
    entry_client_imports: &mut IndexSet<String>,
    from_route: Option<&ReactRoute>,
  ) {
    let data = module
      .as_normal_module()
      .and_then(|m| Some(m.resource_resolved_data()));
    let module_type = module.module_type();
    if let Some(data) = data {
      let resource_path_str = data
        .resource_path
        .as_ref()
        .and_then(|f| f.to_str())
        .expect("TODO:");
      let resource_query = &data.resource_query;
      let resource_query_str = if let Some(query) = resource_query.as_ref() {
        query
      } else {
        ""
      };
      let resource_str = format!("{}{}", resource_path_str, resource_query_str);
      let is_css = match module_type {
        ModuleType::Css | ModuleType::CssModule | ModuleType::CssAuto => true,
        // css asset type only working with experimental.css
        // use filepath match as fallback
        // TODO: maybe we check module.identifier() has css-loader instead
        _ => CSS_RE.is_match(resource_path_str),
      };
      // TODO: check css file is in used
      // TODO: unique css files from other entry
      let is_client_components = match module.build_info() {
        Some(build_info) => has_client_directive(&build_info.directives),
        None => false,
      };
      let route_entry: Option<&ReactRoute> = match self.get_route_entry(&resource_str) {
        Some(route) => Some(route),
        None => from_route,
      };
      if self.is_visited(visited_modules, entry_name, route_entry, &resource_str) {
        return;
      }
      self.mark_visited(visited_modules, entry_name, route_entry, &resource_str);

      if is_client_components || is_css {
        if let Some(route_entry) = route_entry {
          self.insert_client_imports(
            client_imports,
            Some(route_entry.name.as_str()),
            &resource_str,
          )
        } else {
          entry_client_imports.insert(String::from(&resource_str));
        }
      } else {
        let mg = compilation.get_module_graph();
        for connection in mg.get_outgoing_connections(&module.identifier()) {
          let m = mg
            .get_module_by_dependency_id(&connection.dependency_id)
            .expect("should exist");
          self.filter_client_components(
            compilation,
            entry_name,
            m,
            visited_modules,
            client_imports,
            entry_client_imports,
            route_entry,
          );
        }
      }
    }
  }
  fn filter_server_actions(
    &self,
    compilation: &Compilation,
    module: &Box<dyn Module>,
    visited_modules: &mut HashSet<String>,
    entry_server_imports: &mut IndexSet<String>,
  ) {
    let data = module
      .as_normal_module()
      .and_then(|m| Some(m.resource_resolved_data()));
    if let Some(data) = data {
      let resource_path_str = data
        .resource_path
        .as_ref()
        .and_then(|f| f.to_str())
        .expect("TODO:");
      let resource_query = &data.resource_query;
      let resource_query_str = if let Some(query) = resource_query.as_ref() {
        query
      } else {
        ""
      };
      let resource_str = format!("{}{}", resource_path_str, resource_query_str);
      if visited_modules.contains(&resource_str) {
        return;
      }
      visited_modules.insert(resource_str.clone());
      let is_server_action = match module.build_info() {
        Some(build_info) => has_server_directive(&build_info.directives),
        None => false,
      };
      if is_server_action {
        entry_server_imports.insert(String::from(resource_str));
      };
      let mg = compilation.get_module_graph();
      for connection in mg.get_outgoing_connections(&module.identifier()) {
        let m = mg
          .get_module_by_dependency_id(&connection.dependency_id)
          .expect("should exist");
        self.filter_server_actions(compilation, m, visited_modules, entry_server_imports);
      }
    }
  }
}

#[plugin_hook(CompilerFinishMake for RSCClientEntryRspackPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let now = Instant::now();
  // Client imports groupby entry or route chunkname
  let mut client_imports: ClientImports = HashMap::new();
  let mut server_imports: ClientImports = HashMap::new();
  let mut visited_modules: HashMap<String, HashSet<String>> = HashMap::new();
  for (name, entry) in &compilation.entries {
    let mut entry_client_imports: IndexSet<String> = IndexSet::new();
    let mut entry_server_imports: IndexSet<String> = IndexSet::new();
    let mut visited_modules_of_server_actions: HashSet<String> = HashSet::new();
    let mg = compilation.get_module_graph();
    let entry_module = mg
      .get_module_by_dependency_id(&entry.dependencies[0])
      .expect("should exist");
    self.filter_client_components(
      compilation,
      name,
      entry_module,
      &mut visited_modules,
      &mut client_imports,
      &mut entry_client_imports,
      None,
    );
    self.filter_server_actions(
      compilation,
      entry_module,
      &mut visited_modules_of_server_actions,
      &mut entry_server_imports,
    );
    client_imports.insert(String::from(name), entry_client_imports);
    server_imports.insert(String::from(name), entry_server_imports);
  }
  let mut shared_client_imports_guard = SHARED_CLIENT_IMPORTS.write().await;
  *shared_client_imports_guard = client_imports.clone();
  let mut shared_server_imports_guard = SHARED_SERVER_IMPORTS.write().await;
  *shared_server_imports_guard = server_imports.clone();
  // TODO: custom main entry name, all other entries depend on this entry
  // let main_name = "server-entry";
  // let cc = client_imports.clone();
  // let main = cc.get(main_name).unwrap();
  for (name, value) in client_imports.iter_mut() {
    // if name != main_name {
    //   for import in main {
    //     value.shift_remove(import.as_str());
    //   }
    // }
    // Make HMR friendly
    value.sort();
    let output_file = format!("[{}]_client_imports.json", name);
    let content = to_string(&value);

    match content {
      Ok(content) => {
        compilation.assets_mut().insert(
          output_file,
          CompilationAsset {
            source: Some(RawSource::from(content.as_str()).boxed()),
            info: AssetInfo {
              immutable: false,
              version: generate_asset_version(&content),
              ..AssetInfo::default()
            },
          },
        );
      }
      Err(_) => (),
    }
  }
  for (name, value) in server_imports.iter_mut() {
    // Make HMR friendly
    value.sort();
    let output_file = format!("[{}]_server_imports.json", name);
    let content = to_string(&value);
    match content {
      Ok(content) => {
        compilation.assets_mut().insert(
          output_file,
          CompilationAsset {
            source: Some(RawSource::from(content.as_str()).boxed()),
            info: AssetInfo {
              immutable: false,
              version: generate_asset_version(&content),
              ..AssetInfo::default()
            },
          },
        );
      }
      Err(_) => (),
    }
  }
  self.add_entry(compilation).await?;
  tracing::debug!(
    "collect all client & server imports took {} ms.",
    now.elapsed().as_millis()
  );
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for RSCClientEntryRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let plugin: RSCServerReferenceManifest = RSCServerReferenceManifest {};
  plugin.process_assets_stage_optimize_hash(compilation).await
}

#[async_trait]
impl Plugin for RSCClientEntryRspackPlugin {
  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
