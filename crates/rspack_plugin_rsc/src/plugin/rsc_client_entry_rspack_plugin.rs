use std::collections::HashSet;

use async_trait::async_trait;
use rspack_core::{Compilation, Module, Plugin};
use rspack_error::Result;

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
    let resource_path = &data.unwrap().resource_path;
    let resource_path_str = resource_path.to_str().expect("Should exits");
    if visited_modules.contains(resource_path_str) {
      return;
    }
    visited_modules.insert(String::from(resource_path_str));
    let og = module.original_source().unwrap();
    // TODO: detect client components based on directives
    if let Some(_) = og.source().find("use client") {
      collect_client_imports.insert(String::from(resource_path_str));
    } else {
      for connection in compilation.module_graph.get_outgoing_connections(module) {
        let m = compilation
          .module_graph
          .get_module(&connection.dependency_id)
          .expect("should exist");
        self.filter_client_components(compilation, m, visited_modules, collect_client_imports);
      }
    }
  }
}

#[async_trait]
impl Plugin for RSCClientEntryRspackPlugin {
  async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
    let mut visited_modules: HashSet<String> = HashSet::new();
    let mut collected_client_imports: HashSet<String> = HashSet::new();
    for (_, entry) in &compilation.entries {
      let entry_module = compilation
        .module_graph
        .get_module(&entry.dependencies[0])
        .expect("should exist");
      self.filter_client_components(
        compilation,
        entry_module,
        &mut visited_modules,
        &mut collected_client_imports,
      );
    }
    println!("client_imports {:?}", collected_client_imports);
    Ok(())
  }
}
