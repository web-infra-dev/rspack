use rspack_core::{
  BuildInfo, BuildMeta, BuildMetaExportsType, DependencyTemplate, ExportsArgument, ModuleArgument,
  ModuleIdentifier, ModuleType,
};
use rspack_error::internal_error;
use swc_core::ecma::ast::{ArrowExpr, AwaitExpr, Constructor, Function, ModuleItem, Program};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use crate::dependency::HarmonyCompatibilityDependency;

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/HarmonyDetectionParserPlugin.js
pub struct HarmonyDetectionScanner<'a> {
  module_identifier: &'a ModuleIdentifier,
  build_info: &'a mut BuildInfo,
  build_meta: &'a mut BuildMeta,
  module_type: &'a ModuleType,
  code_generable_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  errors: &'a mut Vec<rspack_error::Error>,
}

impl<'a> HarmonyDetectionScanner<'a> {
  pub fn new(
    module_identifier: &'a ModuleIdentifier,
    build_info: &'a mut BuildInfo,
    build_meta: &'a mut BuildMeta,
    module_type: &'a ModuleType,
    code_generable_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    errors: &'a mut Vec<rspack_error::Error>,
  ) -> Self {
    Self {
      module_identifier,
      build_info,
      build_meta,
      module_type,
      code_generable_dependencies,
      errors,
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

    if has_top_level_await(program) {
      if is_harmony || strict_harmony_module {
        self.build_meta.has_await = true;
      } else {
        self.errors.push(internal_error!(
          "Top-level-await is only supported in EcmaScript Modules: {}",
          self.module_identifier
        ));
      }
    }

    if strict_harmony_module {
      self.build_meta.strict_harmony_module = true;
      self.build_meta.module_argument = ModuleArgument::WebpackModule;
    }
  }
}

fn has_top_level_await(m: &Program) -> bool {
  let mut visitor = TopLevelAwaitScanner::default();
  m.visit_with(&mut visitor);
  visitor.has_top_level_await
}

#[derive(Default)]
struct TopLevelAwaitScanner {
  has_top_level_await: bool,
}

impl Visit for TopLevelAwaitScanner {
  noop_visit_type!();
  fn visit_arrow_expr(&mut self, _: &ArrowExpr) {}
  fn visit_constructor(&mut self, _: &Constructor) {}
  fn visit_function(&mut self, _: &Function) {}

  fn visit_await_expr(&mut self, _: &AwaitExpr) {
    self.has_top_level_await = true;
  }
}
