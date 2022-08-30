// mod js_module;
// pub use js_module::*;

use rspack_error::Result;
use std::fmt::Debug;

use rspack_core::{Module, ModuleRenderResult, ModuleType, SourceType};

use swc_css::{ast::Stylesheet, visit::VisitMutWith};

use crate::{visitors::DependencyScanner, SWC_COMPILER};

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

pub struct CssModule {
  pub ast: Stylesheet,
  pub source_type_list: &'static [SourceType; 2],
  pub meta: Option<String>,
}

impl Debug for CssModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CssModule").field("ast", &"...").finish()
  }
}

impl Module for CssModule {
  #[inline(always)]
  fn module_type(&self) -> ModuleType {
    ModuleType::Css
  }

  #[inline(always)]
  fn source_types(
    &self,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> &[SourceType] {
    self.source_type_list.as_ref()
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let result = match requested_source_type {
      SourceType::Css => Some(ModuleRenderResult::Css(SWC_COMPILER.codegen(&self.ast))),
      // This is just a temporary solution for css-modules
      SourceType::JavaScript => Some(ModuleRenderResult::JavaScript(format!(
        r#"function(module, exports, __rspack_require__, __rspack_dynamic_require__) {{
          "use strict";
          {}
        }};
        "#,
        self
          .meta
          .clone()
          .map(|item| { format!("module.exports = {}", item) })
          .unwrap_or_else(|| "".to_string())
      ))),
      _ => None,
    };
    Ok(result)
  }

  fn dependencies(&mut self) -> Vec<rspack_core::ModuleDependency> {
    let mut scanner = DependencyScanner::default();
    self.ast.visit_mut_with(&mut scanner);
    scanner.dependencies
  }
}
