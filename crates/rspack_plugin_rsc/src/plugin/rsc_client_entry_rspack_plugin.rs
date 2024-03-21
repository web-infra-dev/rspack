use std::collections::{HashMap, HashSet};
use std::time::Instant;

use async_trait::async_trait;
use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{AssetInfo, Compilation, CompilationAsset, Module, ModuleType, Plugin};
use rspack_error::Result;
use serde_json::to_string;

#[derive(Debug, Default, Clone)]
pub struct RSCClientEntryRspackPlugin {}

impl RSCClientEntryRspackPlugin {
  pub fn new() -> Self {
    Self {}
  }
  fn filter_client_components(
    &self,
    compilation: &Compilation,
    module: &Box<dyn Module>,
    visited_modules: &mut HashSet<String>,
    collect_client_imports: &mut HashSet<String>,
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
      let use_client = String::from("use client");
      let is_client_components = match module.build_info() {
        Some(build_info) => build_info.directives.contains(&use_client),
        None => false,
      };
      // let og = module.original_source().unwrap();
      if is_client_components || is_css {
        collect_client_imports.insert(String::from(resource_path_str));
      } else {
        let mg = compilation.get_module_graph();
        for connection in mg.get_outgoing_connections(&module.identifier()) {
          let m = mg
            .get_module_by_dependency_id(&connection.dependency_id)
            .expect("should exist");
          self.filter_client_components(compilation, m, visited_modules, collect_client_imports);
        }
      }
    }
  }
}

#[async_trait]
impl Plugin for RSCClientEntryRspackPlugin {
  async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
    let now = Instant::now();
    let mut client_imports: HashMap<String, HashSet<String>> = HashMap::new();
    let mut entrypoints: HashMap<String, String> = HashMap::new();
    for (name, entry) in &compilation.entries {
      let mg = compilation.get_module_graph();
      let entry_module = mg
        .get_module_by_dependency_id(&entry.dependencies[0])
        .expect("should exist");
      let data = entry_module
        .as_normal_module()
        .and_then(|m| Some(m.resource_resolved_data()));
      if let Some(data) = data {
        let resource_path = &data.resource_path;
        let resource_path_str = resource_path.to_str().expect("Should exits");
        entrypoints.insert(name.clone(), String::from(resource_path_str));
      }
    }
    for (name, entry) in &compilation.entries {
      let mut collected_client_imports: HashSet<String> = HashSet::new();
      let mut visited_modules: HashSet<String> = HashSet::new();
      // TODO: should only controlled by spa mod
      for (other_name, other_entry) in &entrypoints {
        if other_name.clone() != name.clone() {
          visited_modules.insert(other_entry.clone());
        }
      }
      let mg = compilation.get_module_graph();
      let entry_module = mg
        .get_module_by_dependency_id(&entry.dependencies[0])
        .expect("should exist");
      self.filter_client_components(
        compilation,
        entry_module,
        &mut visited_modules,
        &mut collected_client_imports,
      );
      client_imports.insert(String::from(name), collected_client_imports);
    }
    for (name, value) in client_imports {
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
}
