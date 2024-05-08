use std::fmt::Write;
use std::hash::Hasher;
use std::path::PathBuf;

use heck::{ToKebabCase, ToLowerCamelCase};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rspack_core::rspack_sources::{ConcatSource, RawSource};
use rspack_core::{
  to_identifier, Compilation, CompilerOptions, Context, GenerateContext, Mode, PathData,
  ResourceData, RuntimeGlobals,
};
use rspack_core::{CssExportsConvention, LocalIdentName};
use rspack_error::{error, Result};
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash};
use rspack_util::identifier::make_paths_relative;
use rspack_util::infallible::ResultInfallibleExt;
use rustc_hash::FxHashSet as HashSet;
use sugar_path::SugarPath;
use swc_core::common::Spanned;
use swc_core::css::modules::CssClassName;
use swc_core::ecma::atoms::Atom;

use crate::parser_and_generator::{CssExport, CssExportsType};

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
pub static AUTO_PUBLIC_PATH_PLACEHOLDER_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(AUTO_PUBLIC_PATH_PLACEHOLDER).expect("Invalid regexp"));
pub static LEADING_DIGIT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\d+").expect("Invalid regexp"));

pub struct ModulesTransformConfig<'a> {
  resource_data: &'a ResourceData,
  local_name_ident: &'a LocalIdentName,
  hash_function: &'a HashFunction,
  hash_digest: &'a HashDigest,
  hash_digest_length: usize,
  hash_salt: &'a HashSalt,
  unique_name: &'a str,
  mode: &'a Mode,
  context: &'a Context,
}

impl<'a> ModulesTransformConfig<'a> {
  pub fn new(
    resource_data: &'a ResourceData,
    local_name_ident: &'a LocalIdentName,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    let output = &compiler_options.output;
    Self {
      resource_data,
      local_name_ident,
      hash_function: &output.hash_function,
      hash_digest: &output.hash_digest,
      hash_digest_length: output.hash_digest_length,
      hash_salt: &output.hash_salt,
      unique_name: &output.unique_name,
      mode: &compiler_options.mode,
      context: &compiler_options.context,
    }
  }
}

impl swc_core::css::modules::TransformConfig for ModulesTransformConfig<'_> {
  fn new_name_for(&self, local: &Atom) -> Atom {
    let mut relative_path = self.resource_data.resource_path.relative(self.context);
    let relative_str = relative_path.to_string_lossy().to_string();

    // abc.js => ./abc.js, align with webpack
    if !relative_path.is_absolute() && !relative_str.starts_with('.') {
      relative_path = PathBuf::from(format!("./{}", relative_str));
    }

    let hash = {
      let mut hasher = RspackHash::with_salt(self.hash_function, self.hash_salt);
      hasher.write(relative_path.to_string_lossy().as_bytes());
      let contains_local = self
        .local_name_ident
        .template
        .template()
        .map(|t| t.contains("[local]"))
        .unwrap_or_default();
      if !contains_local {
        hasher.write(local.as_bytes());
      }
      let hash = hasher.digest(self.hash_digest);
      LEADING_DIGIT_REGEX
        .replace_all(hash.rendered(self.hash_digest_length), "")
        .into_owned()
    };
    let relative_resource =
      make_paths_relative(self.context.as_str(), &self.resource_data.resource);
    LocalIdentNameRenderOptions {
      path_data: PathData::default()
        .filename(&relative_path.to_string_lossy())
        .hash(&hash)
        // TODO: should be moduleId, but we don't have it at parse,
        // and it's lots of work to move css module compile to generator,
        // so for now let's use hash for compatibility.
        .id(if self.mode.is_development() {
          &relative_resource
        } else {
          &hash
        }),
      local,
      unique_name: self.unique_name,
    }
    .render_local_ident_name(self.local_name_ident)
    .into()
  }
}

static ESCAPE_LOCAL_IDENT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[<>:"/\\|?*\.]"#).expect("Invalid regex"));

struct LocalIdentNameRenderOptions<'a> {
  path_data: PathData<'a>,
  local: &'a str,
  unique_name: &'a str,
}

impl LocalIdentNameRenderOptions<'_> {
  pub fn render_local_ident_name(self, local_ident_name: &LocalIdentName) -> String {
    let mut s = local_ident_name
      .template
      .render(self.path_data, None)
      .always_ok();
    s = s.replace("[local]", self.local);
    s = s.replace("[uniqueName]", self.unique_name);

    s = ESCAPE_LOCAL_IDENT_REGEX.replace_all(&s, "_").into_owned();
    s
  }
}

pub(crate) fn export_locals_convention(
  key: &Atom,
  locals_convention: &CssExportsConvention,
) -> Vec<String> {
  let mut res = Vec::with_capacity(3);
  if locals_convention.as_is() {
    res.push(
      serde_json::to_string(&key)
        .unwrap_or_else(|_| panic!("Failed to stringify css modules exports key")),
    );
  }
  if locals_convention.camel_case() {
    res.push(
      serde_json::to_string(&key.to_lower_camel_case())
        .unwrap_or_else(|_| panic!("Failed to stringify css modules exports key into camel case")),
    );
  }
  if locals_convention.dashes() {
    res.push(
      serde_json::to_string(&key.to_kebab_case())
        .unwrap_or_else(|_| panic!("Failed to stringify css modules exports key into dashes")),
    );
  }
  res
}

pub fn stringify_css_modules_exports_elements(elements: &[CssClassName]) -> Vec<CssExport> {
  elements
    .iter()
    .map(|element| match element {
      CssClassName::Local { name } | CssClassName::Global { name } => CssExport(
        serde_json::to_string(&name.value).expect("TODO:"),
        name.span().into(),
        None,
      ),
      CssClassName::Import { name, from } => CssExport(
        serde_json::to_string(&name.value).expect("TODO:"),
        name.span().into(),
        Some(from.to_string()),
      ),
    })
    .collect::<Vec<_>>()
}

pub fn css_modules_exports_to_string(
  exports: &CssExportsType,
  module: &dyn rspack_core::Module,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<String> {
  let mut code = format!(
    "{}(module.exports = {{\n",
    RuntimeGlobals::MAKE_NAMESPACE_OBJECT
  );
  let module_graph = compilation.get_module_graph();
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(|CssExport(name, _, from)| match from {
        None => name.to_owned(),
        Some(from_name) => {
          let from = module
            .get_dependencies()
            .iter()
            .find_map(|id| {
              let dependency = module_graph.dependency_by_id(id);
              let request = if let Some(d) = dependency.and_then(|d| d.as_module_dependency()) {
                Some(d.request())
              } else {
                dependency
                  .and_then(|d| d.as_context_dependency())
                  .map(|d| d.request())
              };
              if let Some(request) = request
                && request == from_name
              {
                return module_graph.module_graph_module_by_dependency_id(id);
              }
              None
            })
            .expect("should have css from module");

          let from = serde_json::to_string(from.id(&compilation.chunk_graph)).expect("TODO:");
          runtime_requirements.insert(RuntimeGlobals::REQUIRE);
          format!("{}({from})[{name}]", RuntimeGlobals::REQUIRE)
        }
      })
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    for item in key {
      writeln!(code, "  {}: {},", item, content).map_err(|e| error!(e.to_string()))?;
    }
  }
  code += "});\n";
  Ok(code)
}

pub fn css_modules_exports_to_concatenate_module_string(
  exports: &CssExportsType,
  module: &dyn rspack_core::Module,
  generate_context: &mut GenerateContext,
  concate_source: &mut ConcatSource,
) -> Result<()> {
  let GenerateContext {
    compilation,
    concatenation_scope,
    ..
  } = generate_context;
  let Some(ref mut scope) = concatenation_scope else {
    return Ok(());
  };
  let module_graph = compilation.get_module_graph();
  let mut used_identifiers = HashSet::default();
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(|CssExport(name, _span, from)| match from {
        None => name.to_owned(),
        Some(from_name) => {
          let from = module
            .get_dependencies()
            .iter()
            .find_map(|id| {
              let dependency = module_graph.dependency_by_id(id);
              let request = if let Some(d) = dependency.and_then(|d| d.as_module_dependency()) {
                Some(d.request())
              } else {
                dependency
                  .and_then(|d| d.as_context_dependency())
                  .map(|d| d.request())
              };
              if let Some(request) = request
                && request == from_name
              {
                return module_graph.module_graph_module_by_dependency_id(id);
              }
              None
            })
            .expect("should have css from module");

          let from = serde_json::to_string(from.id(&compilation.chunk_graph)).expect("TODO:");
          format!("{}({from})[{name}]", RuntimeGlobals::REQUIRE)
        }
      })
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    for k in key {
      let normalized_k = k.as_str()[1..k.len() - 1].to_owned();
      let mut identifier = to_identifier(&normalized_k);
      let mut i = 0;
      while used_identifiers.contains(&identifier) {
        identifier = format!("{k}{i}");
        i += 1;
      }
      // TODO: conditional support `const or var` after we finished runtimeTemplate utils
      concate_source.add(RawSource::from(format!("var {identifier} = {content};\n")));
      used_identifiers.insert(identifier.clone());
      scope.register_export(k.as_str()[1..k.as_str().len() - 1].into(), identifier);
    }
  }
  Ok(())
}

static STRING_MULTILINE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\\[\n\r\f]").expect("Invalid RegExp"));

static TRIM_WHITE_SPACES: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(^[ \t\n\r\f]*|[ \t\n\r\f]*$)").expect("Invalid RegExp"));

static UNESCAPE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\\([0-9a-fA-F]{1,6}[ \t\n\r\f]?|[\s\S])").expect("Invalid RegExp"));

static DATA: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?i)data:").expect("Invalid RegExp"));

pub fn normalize_url(s: &str) -> String {
  let result = STRING_MULTILINE.replace_all(s, "");
  let result = TRIM_WHITE_SPACES.replace_all(&result, "");
  let result = UNESCAPE.replace_all(&result, |caps: &Captures| {
    caps
      .get(0)
      .and_then(|m| {
        let m = m.as_str();
        if m.len() > 2 {
          if let Ok(r_u32) = u32::from_str_radix(m[1..].trim(), 16) {
            if let Some(ch) = char::from_u32(r_u32) {
              return Some(format!("{}", ch));
            }
          }
          None
        } else {
          Some(m[1..2].to_string())
        }
      })
      .unwrap_or(caps[0].to_string())
  });

  if DATA.is_match(&result) {
    return result.to_string();
  }
  if result.contains('%') {
    if let Ok(r) = urlencoding::decode(&result) {
      return r.to_string();
    }
  }

  result.to_string()
}
