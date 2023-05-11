use rspack_core::{BuildInfo, BuildMeta, BuildMetaExportsType, Dependency, ModuleType};
use swc_core::ecma::ast::{ModuleItem, Program};
use swc_core::ecma::visit::{noop_visit_type, Visit};

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/HarmonyDetectionParserPlugin.js
pub struct HarmonyDetectionScanner<'a> {
  pub build_info: &'a mut BuildInfo,
  pub build_meta: &'a mut BuildMeta,
  pub module_type: &'a ModuleType,
  pub presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
}

impl<'a> HarmonyDetectionScanner<'a> {
  pub fn new(
    build_info: &'a mut BuildInfo,
    build_meta: &'a mut BuildMeta,
    module_type: &'a ModuleType,
    presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
  ) -> Self {
    Self {
      build_info,
      build_meta,
      module_type,
      presentational_dependencies,
    }
  }
}

impl Visit for HarmonyDetectionScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &'_ Program) {
    let strict_harmony_module = self.module_type.is_js_esm();

    let is_harmony = matches!(program, Program::Module(module) if module.body.iter().any(|s| matches!(s, ModuleItem::ModuleDecl(_))));

    if is_harmony || strict_harmony_module {
      self.build_meta.esm = true;
      self.build_meta.exports_type = BuildMetaExportsType::Namespace;
      self.build_info.strict = true;
      self.build_meta.exports_argument = "__webpack_exports__";
    }

    if strict_harmony_module {
      self.build_meta.strict_harmony_module = true;
      self.build_meta.module_argument = "__webpack_module__";
    }
  }
}
