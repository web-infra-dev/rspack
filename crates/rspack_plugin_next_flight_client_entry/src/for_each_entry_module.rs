use rspack_core::{Compilation, DependenciesBlock, EntryData, ModuleGraph, NormalModule};

pub struct EntryModuleIter<'a> {
  module_graph: &'a ModuleGraph<'a>,
  entries_iter: indexmap::map::Iter<'a, String, EntryData>,
}

impl<'a> EntryModuleIter<'a> {
  fn new(compilation: &'a Compilation, module_graph: &'a ModuleGraph<'a>) -> Self {
    Self {
      module_graph,
      entries_iter: compilation.entries.iter(),
    }
  }
}

impl<'a> Iterator for EntryModuleIter<'a> {
  type Item = (&'a String, &'a NormalModule);

  fn next(&mut self) -> Option<Self::Item> {
    let module_graph = &self.module_graph;

    loop {
      if let Some((name, entry)) = self.entries_iter.next() {
        // Skip for entries under pages/
        if name.starts_with("pages/") {
          continue;
        }

        // Check if the page entry is a server component or not.
        let Some(dependency_id) = entry.dependencies.get(0) else {
          continue;
        };

        // Ensure only next-app-loader entries are handled.
        let Some(entry_dependency) = module_graph.dependency_by_id(dependency_id) else {
          continue;
        };
        let Some(entry_dependency) = entry_dependency.as_module_dependency() else {
          continue;
        };

        let request = entry_dependency.request();

        if !request.starts_with("next-edge-ssr-loader?")
          && !request.starts_with("next-edge-app-route-loader?")
          && !(request.starts_with(&format!("{}?", "next-app-loader"))
            || request.starts_with(&format!("{}?", "builtin:next-app-loader")))
        {
          continue;
        }

        let Some(entry_module) = module_graph.get_resolved_module(dependency_id) else {
          continue;
        };
        let Some(mut entry_module) = entry_module.as_normal_module() else {
          continue;
        };

        if request.starts_with("next-edge-ssr-loader?")
          || request.starts_with("next-edge-app-route-loader?")
        {
          for dependency_id in entry_module.get_dependencies() {
            let Some(dependency) = module_graph.dependency_by_id(dependency_id) else {
              continue;
            };
            let Some(dependency) = dependency.as_module_dependency() else {
              continue;
            };
            if dependency.request().contains("next-app-loader") {
              if let Some(module) = module_graph.get_resolved_module(dependency_id) {
                if let Some(module) = module.as_normal_module() {
                  entry_module = module;
                }
              }
            }
          }
        }

        return Some((name, entry_module));
      } else {
        return None;
      }
    }
  }
}

pub fn for_each_entry_module<'a>(
  compilation: &'a Compilation,
  module_graph: &'a ModuleGraph<'a>,
) -> EntryModuleIter<'a> {
  EntryModuleIter::new(compilation, module_graph)
}
