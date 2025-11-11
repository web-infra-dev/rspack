use rspack_core::{Compilation, DependenciesBlock, EntryData, ModuleGraph, NormalModule};

pub struct ServerEntryModules<'a> {
  module_graph: &'a ModuleGraph<'a>,
  entries_iter: indexmap::map::Iter<'a, String, EntryData>,
}

impl<'a> ServerEntryModules<'a> {
  pub fn new(compilation: &'a Compilation, module_graph: &'a ModuleGraph<'a>) -> Self {
    Self {
      module_graph,
      entries_iter: compilation.entries.iter(),
    }
  }
}

impl<'a> Iterator for ServerEntryModules<'a> {
  type Item = &'a NormalModule;

  fn next(&mut self) -> Option<Self::Item> {
    todo!()
  }
}
