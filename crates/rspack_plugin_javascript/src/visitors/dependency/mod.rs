mod code_generation;
mod harmony_detection;
mod hmr_scanner;
mod scanner;
mod util;
pub use code_generation::*;
pub use harmony_detection::*;
use rspack_core::{
  ast::javascript::Program, CompilerOptions, Dependency, ModuleDependency, ModuleType, ResourceData,
};
use swc_core::common::Mark;
pub use util::*;

use self::{hmr_scanner::HmrDependencyScanner, scanner::DependencyScanner};

pub type ScanDependenciesResult = (Vec<Box<dyn ModuleDependency>>, Vec<Box<dyn Dependency>>);

pub fn scan_dependencies(
  program: &Program,
  unresolved_mark: Mark,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  module_type: &ModuleType,
) -> ScanDependenciesResult {
  let mut dependencies: Vec<Box<dyn ModuleDependency>> = vec![];
  let mut presentational_dependencies: Vec<Box<dyn Dependency>> = vec![];
  program.visit_with(&mut HarmonyDetection::new(
    module_type,
    &mut presentational_dependencies,
  ));
  program.visit_with_path(
    &mut DependencyScanner::new(
      unresolved_mark,
      resource_data,
      compiler_options,
      &mut dependencies,
      &mut presentational_dependencies,
    ),
    &mut Default::default(),
  );
  program.visit_with_path(
    &mut HmrDependencyScanner::new(&mut dependencies),
    &mut Default::default(),
  );
  (dependencies, presentational_dependencies)
}
