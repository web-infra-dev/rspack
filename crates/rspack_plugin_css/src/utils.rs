use std::{
  fmt::Write,
  hash::{Hash, Hasher},
  path::Path,
};

use data_encoding::{Encoding, Specification};
use heck::{ToKebabCase, ToLowerCamelCase};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{Compilation, ModuleDependency, PathData, RuntimeGlobals};
use rspack_error::{internal_error, Result};
use swc_core::css::modules::CssClassName;
use swc_core::ecma::atoms::JsWord;
use xxhash_rust::xxh3::Xxh3;

use crate::plugin::{LocalIdentName, LocalIdentNameRenderOptions, LocalsConvention};

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
pub static AUTO_PUBLIC_PATH_PLACEHOLDER_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(AUTO_PUBLIC_PATH_PLACEHOLDER).expect("Invalid regexp"));

static ENCODER: Lazy<Encoding> = Lazy::new(|| {
  let mut spec = Specification::new();
  spec
    .symbols
    .push_str("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_-");
  spec.encoding().expect("Invalid specification")
});

pub struct ModulesTransformConfig<'a> {
  pub filename: &'a Path,
  pub local_name_ident: &'a LocalIdentName,
}

impl swc_core::css::modules::TransformConfig for ModulesTransformConfig<'_> {
  fn new_name_for(&self, local: &JsWord) -> JsWord {
    let hash = {
      let mut hasher = Xxh3::default();
      self.filename.hash(&mut hasher);
      local.hash(&mut hasher);
      let hash = hasher.finish();
      let hash = ENCODER.encode(&hash.to_le_bytes());
      if hash.as_bytes()[0].is_ascii_digit() {
        format!("_{hash}")
      } else {
        hash
      }
    };
    self
      .local_name_ident
      .render(LocalIdentNameRenderOptions {
        path_data: PathData::default().filename(self.filename).hash(&hash),
        local: Some(local),
      })
      .into()
  }
}

pub fn css_modules_exports_to_string(
  exports: &IndexMap<JsWord, Vec<CssClassName>>,
  module: &dyn rspack_core::Module,
  compilation: &Compilation,
  locals_convention: &LocalsConvention,
) -> Result<String> {
  let mut code = String::from("module.exports = {\n");
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(|element| match element {
        CssClassName::Local { name } | CssClassName::Global { name } => {
          serde_json::to_string(&name.value).expect("TODO:")
        }
        CssClassName::Import { name, from } => {
          let name = serde_json::to_string(&name.value).expect("TODO:");

          let from = compilation
            .module_graph
            .module_graph_module_by_identifier(&module.identifier())
            .and_then(|mgm| {
              // workaround
              mgm.dependencies.iter().find_map(|id| {
                let dependency = compilation.module_graph.dependency_by_id(id);
                if let Some(dependency) = dependency {
                  if dependency.request() == from {
                    return compilation
                      .module_graph
                      .module_graph_module_by_dependency_id(id);
                  }
                }
                None
              })
            })
            .expect("should have css from module");

          let from = serde_json::to_string(from.id(&compilation.chunk_graph)).expect("TODO:");
          format!("{}({from})[{name}]", RuntimeGlobals::REQUIRE)
        }
      })
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    if locals_convention.as_is() {
      writeln!(
        code,
        "  {}: {},",
        serde_json::to_string(&key).expect("TODO:"),
        content,
      )
      .map_err(|e| internal_error!(e.to_string()))?;
    }
    if locals_convention.camel_case() {
      writeln!(
        code,
        "  {}: {},",
        serde_json::to_string(&key.to_lower_camel_case()).expect("TODO:"),
        content,
      )
      .map_err(|e| internal_error!(e.to_string()))?;
    }
    if locals_convention.dashes() {
      writeln!(
        code,
        "  {}: {},",
        serde_json::to_string(&key.to_kebab_case()).expect("TODO:"),
        content,
      )
      .map_err(|e| internal_error!(e.to_string()))?;
    }
  }
  code += "};\n";
  Ok(code)
}
