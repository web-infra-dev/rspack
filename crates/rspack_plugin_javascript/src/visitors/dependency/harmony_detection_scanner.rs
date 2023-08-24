use rspack_core::{
  BuildInfo, BuildMeta, BuildMetaExportsType, DependencyTemplate, ExportsArgument, ModuleArgument,
  ModuleType,
};
use swc_core::ecma::ast::{ModuleItem, Program};
use swc_core::ecma::visit::{noop_visit_type, Visit};

use crate::dependency::HarmonyCompatibilityDependency;

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/HarmonyDetectionParserPlugin.js
pub struct HarmonyDetectionScanner<'a> {
  build_info: &'a mut BuildInfo,
  build_meta: &'a mut BuildMeta,
  module_type: &'a ModuleType,
  code_generable_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
}

impl<'a> HarmonyDetectionScanner<'a> {
  pub fn new(
    build_info: &'a mut BuildInfo,
    build_meta: &'a mut BuildMeta,
    module_type: &'a ModuleType,
    code_generable_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  ) -> Self {
    Self {
      build_info,
      build_meta,
      module_type,
      code_generable_dependencies,
    }
  }
}

impl Visit for HarmonyDetectionScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &'_ Program) {
    let strict_harmony_module = matches!(self.module_type, ModuleType::JsEsm | ModuleType::JsxEsm);

    let is_harmony = matches!(program, Program::Module(module) if module.body.iter().any(|s| matches!(s, ModuleItem::ModuleDecl(_))));

    if is_harmony || strict_harmony_module {
      self
        .code_generable_dependencies
        .push(Box::new(HarmonyCompatibilityDependency {}));
      self.build_meta.esm = true;
      self.build_meta.exports_type = BuildMetaExportsType::Namespace;
      self.build_info.strict = true;
      self.build_meta.exports_argument = ExportsArgument::WebpackExports;
    }

    if strict_harmony_module {
      self.build_meta.strict_harmony_module = true;
      self.build_meta.module_argument = ModuleArgument::WebpackModule;
    }
  }
}
