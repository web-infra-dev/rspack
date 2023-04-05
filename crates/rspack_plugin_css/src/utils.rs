use std::{
  fmt::Write,
  hash::{Hash, Hasher},
};

use data_encoding::{Encoding, Specification};
use heck::{ToKebabCase, ToLowerCamelCase};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rspack_core::{Compilation, ModuleDependency, RuntimeGlobals};
use rspack_error::{internal_error, Result};
use swc_core::css::modules::CssClassName;
use swc_core::ecma::atoms::JsWord;
use xxhash_rust::xxh3::Xxh3;

use crate::plugin::{LocalIdentName, LocalIdentNameRenderOptions, LocalsConvention};

static ENCODER: Lazy<Encoding> = Lazy::new(|| {
  let mut spec = Specification::new();
  spec
    .symbols
    .push_str("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_-");
  spec.encoding().expect("Invalid specification")
});

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
            let hash = hasher.finish();
            let hash = ENCODER.encode(&hash.to_le_bytes());
            if hash.as_bytes()[0].is_ascii_digit() {
              format!("_{hash}")
            } else {
              hash
            }
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
    let mut class_names = elements
      .iter()
      .map(|element| match element {
        CssClassName::Local { name } | CssClassName::Global { name } => {
          serde_json::to_string(&format!("{name}")).expect("TODO:")
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
          format!("{}({from})[{name}]", RuntimeGlobals::REQUIRE)
        }
      })
      .collect::<Vec<_>>();
    let mut n = 1;
    // insert space between each classname
    while n < class_names.len() {
      class_names.insert(n, String::from("\" \""));
      n += 2;
    }
    let content = class_names.join(" + ");
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
