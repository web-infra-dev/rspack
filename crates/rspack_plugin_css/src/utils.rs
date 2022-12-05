use indexmap::IndexMap;
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
  exports: IndexMap<JsWord, Vec<CssClassName>>,
) -> Result<String> {
  use std::fmt::Write;
  let mut code = String::from("{\n");
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(|element| match element {
        CssClassName::Local { name } | CssClassName::Global { name } => name,
        CssClassName::Import { .. } => "TODO",
      })
      .collect::<Vec<_>>()
      .join(" ");
    writeln!(
      code,
      "  {}: {},",
      serde_json::to_string(&key).unwrap(),
      serde_json::to_string(&content).unwrap(),
    )
    .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?;
  }
  code += "}";
  Ok(code)
}
