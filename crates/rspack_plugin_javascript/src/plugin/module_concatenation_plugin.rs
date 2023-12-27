use std::cmp::Ordering;
use std::sync::Arc;

use rspack_core::{Compilation, ModuleIdentifier};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
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

struct RuntimeSpec;

impl<'a> ConcatConfiguration<'a> {
  fn new(root_module: ModuleIdentifier, runtime: &'a RuntimeSpec) -> Self {
    let mut modules = HashSet::new();
    modules.insert(root_module);

    ConcatConfiguration {
      root_module,
      runtime,
      modules,
      warnings: HashMap::new(),
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

struct ModuleConcatenationPlugin;

impl ModuleConcatenationPlugin {
  fn get_imports(
    compilation: &Compilation,
    mi: ModuleIdentifier,
    runtime: RuntimeSpec,
  ) -> HashSet<ModuleIdentifier> {
    let mg = &compilation.module_graph;
    let mut set = HashSet::default();
    let module = mg.module_by_identifier(&mi).expect("should have module");
    for d in module.get_dependencies() {}
  }
}
