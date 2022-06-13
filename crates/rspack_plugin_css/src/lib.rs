// mod js_module;
// pub use js_module::*;

use rspack_core::{
  BoxModule, JobContext, Module, ModuleDependency, ParseModuleArgs, Plugin, ResolveKind, SourceType,
};

#[derive(Debug)]
pub struct CssPlugin {}

#[derive(Debug)]
struct CssModule {
  pub source: String,
}

impl Module for CssModule {
  fn render(
    &self,
    module: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> String {
    format!("export default JSON.parse(`{}`)", self.source)
  }

  fn dependencies(&mut self) -> Vec<rspack_core::ModuleDependency> {
    if self.source.contains("dep.css") {
      vec![ModuleDependency {
        specifier: "dep.css".to_string(),
        kind: ResolveKind::AtImport,
      }]
    } else {
      vec![]
    }
  }
}

impl Plugin for CssPlugin {
  fn register_parse_module(&self, _ctx: rspack_core::PluginContext) -> Option<Vec<SourceType>> {
    Some(vec![SourceType::Css])
  }

  fn parse_module(
    &self,
    _ctx: rspack_core::PluginContext<&mut JobContext>,
    args: ParseModuleArgs,
  ) -> BoxModule {
    Box::new(CssModule {
      source: args.source,
    })
  }
}
