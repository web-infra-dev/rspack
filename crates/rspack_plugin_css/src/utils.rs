use std::fmt::Write;

use indexmap::IndexMap;
use rspack_core::{
  runtime_globals::REQUIRE, Compilation, Dependency, ModuleDependency, ResolveKind,
};
use rspack_error::Result;
use swc_core::ecma::atoms::JsWord;
use swc_css::modules::CssClassName;

pub struct ModulesTransformConfig {
  pub suffix: String,
}

impl swc_css::modules::TransformConfig for ModulesTransformConfig {
  fn new_name_for(&self, local: &JsWord) -> JsWord {
    format!("{}{}", *local, self.suffix).into()
  }
}

pub fn css_modules_exports_to_string(
  exports: &IndexMap<JsWord, Vec<CssClassName>>,
  module: &dyn rspack_core::Module,
  compilation: &Compilation,
) -> Result<String> {
  let mut code = String::from("{\n");
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(|element| match element {
        CssClassName::Local { name } | CssClassName::Global { name } => {
          serde_json::to_string(&format!("{name} ")).unwrap()
        }
        CssClassName::Import { name, from } => {
          let name = serde_json::to_string(name).unwrap();
          let from = Dependency {
            parent_module_identifier: Some(module.identifier().into()),
            detail: ModuleDependency {
              specifier: from.to_string(),
              kind: ResolveKind::AtImport,
              span: None,
            },
          };
          let from = compilation
            .module_graph
            .module_by_dependency(&from)
            .expect("should have css from module");
          let from = serde_json::to_string(&from.id).unwrap();
          format!("{REQUIRE}({from})[{name}]")
        }
      })
      .collect::<Vec<_>>()
      .join(" + ");
    writeln!(
      code,
      "  {}: {},",
      serde_json::to_string(&key).unwrap(),
      content,
    )
    .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?;
  }
  code += "}";
  Ok(code)
}
