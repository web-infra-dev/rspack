use std::{
  fmt::Write,
  hash::{Hash, Hasher},
};

use heck::{ToKebabCase, ToLowerCamelCase};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{runtime_globals::REQUIRE, Compilation, ModuleDependency};
use rspack_error::{internal_error, Result};
use swc_core::css::modules::CssClassName;
use swc_core::ecma::atoms::JsWord;
use xxhash_rust::xxh3::Xxh3;

use crate::plugin::{LocalIdentName, LocalIdentNameRenderOptions, LocalsConvention};

static LEADING_DIGITS: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+").expect("Invalid regex"));
static NOT_ALPHANUMERIC_OR_UNDERSCORE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[^A-Za-z0-9_]+").expect("Invalid regex"));

pub struct ModulesTransformConfig<'l> {
  pub name: Option<String>,
  pub path: Option<String>,
  pub ext: Option<String>,
  pub local_name_ident: &'l LocalIdentName,
}

impl swc_core::css::modules::TransformConfig for ModulesTransformConfig<'_> {
  fn new_name_for(&self, local: &JsWord) -> JsWord {
    self
      .local_name_ident
      .render(LocalIdentNameRenderOptions {
        filename_options: rspack_core::FilenameRenderOptions {
          name: self.name.clone(),
          path: self.path.clone(),
          extension: self.ext.clone(),
          hash: Some({
            let mut hasher = Xxh3::default();
            self.name.hash(&mut hasher);
            self.path.hash(&mut hasher);
            self.ext.hash(&mut hasher);
            local.hash(&mut hasher);
            let hash = format!("{:x}", hasher.finish());
            let hash = LEADING_DIGITS.replace(&hash, "");
            let hash = NOT_ALPHANUMERIC_OR_UNDERSCORE.replace(&hash, "");
            hash.into_owned()
          }),
          ..Default::default()
        },
        local: Some(local.to_string()),
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
          serde_json::to_string(&format!("{name} ")).expect("TODO:")
        }
        CssClassName::Import { name, from } => {
          let name = serde_json::to_string(name).expect("TODO:");

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
          format!("{REQUIRE}({from})[{name}]")
        }
      })
      .collect::<Vec<_>>()
      .join(" + ");
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
