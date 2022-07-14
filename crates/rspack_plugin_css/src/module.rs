// mod js_module;
// pub use js_module::*;

use anyhow::Result;
use hashbrown::HashSet;
use std::fmt::Debug;

use rspack_core::{Module, ModuleRenderResult, ModuleType, SourceType};

use swc_css::{ast::Stylesheet, visit::VisitMutWith};

use crate::{visitors::DependencyScanner, SWC_COMPILER};

pub struct CssModule {
  pub ast: Stylesheet,
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
  ) -> HashSet<SourceType> {
    HashSet::from_iter([SourceType::JavaScript, SourceType::Css])
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let result = match requested_source_type {
      SourceType::Css => Some(ModuleRenderResult::Css(SWC_COMPILER.codegen(&self.ast))),
      SourceType::JavaScript => Some(ModuleRenderResult::JavaScript(format!(
        r#"rs.define("{}", function(__rspack_require__, module, exports) {{
  "use strict";
}});
"#,
        module.id
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
