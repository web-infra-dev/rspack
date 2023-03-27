//! HarmonyDetectionParserPlugin

use rspack_core::{Dependency, ModuleType};
use swc_core::ecma::{
  ast::{ModuleDecl, ModuleItem, Program},
  visit::Visit,
};

use crate::dependency::HarmonyCompatibilityDependency;

pub struct HarmonyDetection<'t> {
  module_type: &'t ModuleType,
  presentational_dependencies: &'t mut Vec<Box<dyn Dependency>>,
}

impl<'t> HarmonyDetection<'t> {
  pub fn new(
    module_type: &'t ModuleType,
    presentational_dependencies: &'t mut Vec<Box<dyn Dependency>>,
  ) -> Self {
    Self {
      module_type,
      presentational_dependencies,
    }
  }
}

impl Visit for HarmonyDetection<'_> {
  fn visit_program(&mut self, node: &Program) {
    let is_harmony = matches!(self.module_type, ModuleType::JsEsm)
      || matches!(
        node,
        Program::Module(m) if m.body.iter().any(|s| matches!(
          s,
          ModuleItem::ModuleDecl(d) if matches!(
            d,
            ModuleDecl::Import(_) | ModuleDecl::ExportDefaultDecl(_) | ModuleDecl::ExportNamed(_) | ModuleDecl::ExportAll(_))
          )
        )
      );
    if is_harmony {
      self
        .presentational_dependencies
        .push(Box::new(HarmonyCompatibilityDependency::new()))
    }
  }
}
