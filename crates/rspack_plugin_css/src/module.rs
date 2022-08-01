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
    compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let namespace = &compilation.options.output.namespace;
    let result = match requested_source_type {
      SourceType::Css => Some(ModuleRenderResult::Css(SWC_COMPILER.codegen(&self.ast))),
      // SourceType::Css => Some(ModuleRenderResult::Css("1".to_string())),
      SourceType::JavaScript => Some(ModuleRenderResult::JavaScript(format!(
        r#"self["{}"].__rspack_register__(["{}"], {{"{}": function(module, exports, __rspack_require__, __rspack_dynamic_require__) {{
  "use strict";
}}}});
"#,
        namespace, module.id, module.id
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
