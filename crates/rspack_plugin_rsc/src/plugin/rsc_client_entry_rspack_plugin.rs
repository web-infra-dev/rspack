use std::collections::{HashMap, HashSet};
use std::time::Instant;

use async_trait::async_trait;
use indexmap::set::IndexSet;
use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{
  ApplyContext, AssetInfo, Compilation, CompilationAsset, CompilerFinishMake, CompilerOptions,
  Module, ModuleType, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde_json::to_string;

use crate::utils::has_client_directive;

#[derive(Debug, Clone)]
pub struct ReactRoute {
  pub name: String,
  pub import: String,
}

#[derive(Debug, Default, Clone)]
pub struct RSCClientEntryRspackPluginOptions {
  pub routes: Vec<ReactRoute>,
}

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct RSCClientEntryRspackPlugin {
  pub options: RSCClientEntryRspackPluginOptions,
}

impl RSCClientEntryRspackPlugin {
  pub fn new(options: RSCClientEntryRspackPluginOptions) -> Self {
    Self::new_inner(options)
  }
  fn insert_client_imports(
    &self,
    client_imports: &mut HashMap<String, IndexSet<String>>,
    name: Option<&str>,
    client_import: &str,
  ) {
    if let Some(name) = name {
      if client_imports.get(name).is_none() {
        client_imports.insert(String::from(name), IndexSet::default());
      }
      let named_client_imports = client_imports.get_mut(name).unwrap();
      named_client_imports.insert(String::from(client_import));
    }
  }
  fn get_route_entry(&self, resource: &str) -> Option<&ReactRoute> {
    self.options.routes.iter().find(|&f| f.import == resource)
  }
  fn filter_client_components(
    &self,
    compilation: &Compilation,
    module: &Box<dyn Module>,
    visited_modules: &mut HashSet<String>,
    client_imports: &mut HashMap<String, IndexSet<String>>,
    entry_client_imports: &mut IndexSet<String>,
    from_route: Option<&ReactRoute>,
  ) {
    let data = module
      .as_normal_module()
      .and_then(|m| Some(m.resource_resolved_data()));
    let module_type = module.module_type();
    if let Some(data) = data {
      let resource_path = &data.resource_path;
      let resource_path_str = resource_path.to_str().expect("Should exits");
      if visited_modules.contains(resource_path_str) {
        return;
      }
      visited_modules.insert(String::from(resource_path_str));
      let is_css = match module_type {
        ModuleType::Css | ModuleType::CssModule | ModuleType::CssAuto => true,
        _ => false,
      };
      // TODO: check css file is in used
      // TODO: unique css files from other entry
      let is_client_components = match module.build_info() {
        Some(build_info) => has_client_directive(&build_info.directives),
        None => false,
      };
      let route_entry: Option<&ReactRoute> = match self.get_route_entry(resource_path_str) {
        Some(route) => Some(route),
        None => from_route,
      };
      if is_client_components || is_css {
        if let Some(route_entry) = route_entry {
          self.insert_client_imports(
            client_imports,
            Some(route_entry.name.as_str()),
            resource_path_str,
          )
        } else {
          entry_client_imports.insert(String::from(resource_path_str));
        }
      } else {
        let mg = compilation.get_module_graph();
        for connection in mg.get_outgoing_connections(&module.identifier()) {
          let m = mg
            .get_module_by_dependency_id(&connection.dependency_id)
            .expect("should exist");
          self.filter_client_components(
            compilation,
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
}

#[plugin_hook(CompilerFinishMake for RSCClientEntryRspackPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let now = Instant::now();
  // println!("rsc options {:?}", self.options);
  let mut client_imports: HashMap<String, IndexSet<String>> = HashMap::new();
  for (name, entry) in &compilation.entries {
    let mut entry_client_imports: IndexSet<String> = IndexSet::new();
    let mut visited_modules: HashSet<String> = HashSet::new();
    let mg = compilation.get_module_graph();
    let entry_module = mg
      .get_module_by_dependency_id(&entry.dependencies[0])
      .expect("should exist");
    self.filter_client_components(
      compilation,
      entry_module,
      &mut visited_modules,
      &mut client_imports,
      &mut entry_client_imports,
      None,
    );
    client_imports.insert(String::from(name), entry_client_imports);
  }
  // TODO: custom main entry name, all other entries depend on this entry
  let main_name = "server-entry";
  let cc = client_imports.clone();
  let main = cc.get(main_name).unwrap();
  for (name, value) in client_imports.iter_mut() {
    // if name != main_name {
    //   for import in main {
    //     value.shift_remove(import.as_str());
    //   }
    // }
    // Make HMR friendly
    value.sort();
    let content = to_string(&value);
    match content {
      Ok(content) => {
        compilation.assets_mut().insert(
          format!("[{}]_client_imports.json", name),
          CompilationAsset {
            source: Some(RawSource::from(content).boxed()),
            info: AssetInfo {
              immutable: false,
              ..AssetInfo::default()
            },
          },
        );
      }
      Err(_) => (),
    }
  }
  tracing::debug!(
    "collect all client imports took {} ms.",
    now.elapsed().as_millis()
  );
  Ok(())
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
    Ok(())
  }
}
