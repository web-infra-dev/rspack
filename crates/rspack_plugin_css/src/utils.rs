use std::fmt::Write;
use std::hash::Hasher;

use heck::{ToKebabCase, ToLowerCamelCase};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rspack_core::rspack_sources::{ConcatSource, RawSource};
use rspack_core::{
  to_identifier, Compilation, CompilerOptions, GenerateContext, PathData, ResourceData,
  RuntimeGlobals,
};
use rspack_core::{CssExportsConvention, LocalIdentName};
use rspack_error::{error, miette::Diagnostic, Result, TraceableError};
use rspack_error::{miette::Severity, DiagnosticExt};
use rspack_hash::RspackHash;
use rspack_util::identifier::make_paths_relative;
use rspack_util::infallible::ResultInfallibleExt;
use rspack_util::json_stringify;
use rustc_hash::FxHashSet as HashSet;

use crate::parser_and_generator::{CssExport, CssExports};

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
pub static AUTO_PUBLIC_PATH_PLACEHOLDER_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(AUTO_PUBLIC_PATH_PLACEHOLDER).expect("Invalid regexp"));
pub static LEADING_DIGIT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\d+").expect("Invalid regexp"));

#[derive(Debug, Clone)]
pub struct LocalIdentOptions<'a> {
  relative_path: String,
  local_name_ident: &'a LocalIdentName,
  compiler_options: &'a CompilerOptions,
}

impl<'a> LocalIdentOptions<'a> {
  pub fn new(
    resource_data: &ResourceData,
    local_name_ident: &'a LocalIdentName,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    let relative_path = make_paths_relative(
      &compiler_options.context,
      &resource_data.resource_path.to_string_lossy(),
    );
    Self {
      relative_path,
      local_name_ident,
      compiler_options,
    }
  }

  pub fn get_local_ident(&self, local: &str) -> String {
    let output = &self.compiler_options.output;
    let hash = {
      let mut hasher = RspackHash::with_salt(&output.hash_function, &output.hash_salt);
      hasher.write(self.relative_path.as_bytes());
      let contains_local = self
        .local_name_ident
        .template
        .template()
        .map(|t| t.contains("[local]"))
        .unwrap_or_default();
      if !contains_local {
        hasher.write(local.as_bytes());
      }
      let hash = hasher.digest(&output.hash_digest);
      LEADING_DIGIT_REGEX
        .replace_all(hash.rendered(output.hash_digest_length), "")
        .into_owned()
    };
    LocalIdentNameRenderOptions {
      path_data: PathData::default()
        .filename(&self.relative_path)
        .hash(&hash)
        // TODO: should be moduleId, but we don't have it at parse,
        // and it's lots of work to move css module compile to generator,
        // so for now let's use hash for compatibility.
        .id(if self.compiler_options.mode.is_development() {
          &self.relative_path
        } else {
          &hash
        }),
      local,
      unique_name: &output.unique_name,
    }
    .render_local_ident_name(&self.local_name_ident)
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
  key: &str,
  locals_convention: &CssExportsConvention,
) -> Vec<String> {
  let mut res = Vec::with_capacity(3);
  if locals_convention.as_is() {
    res.push(key.to_string());
  }
  if locals_convention.camel_case() {
    res.push(key.to_lower_camel_case());
  }
  if locals_convention.dashes() {
    res.push(key.to_kebab_case());
  }
  res
}

pub fn css_modules_exports_to_string(
  exports: &CssExports,
  module: &dyn rspack_core::Module,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  ns_obj: &str,
  left: &str,
  right: &str,
) -> Result<String> {
  let mut code = format!("{}{}module.exports = {{\n", ns_obj, left);
  let module_graph = compilation.get_module_graph();
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(|CssExport { ident, from }| match from {
        None => json_stringify(ident),
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
          format!(
            "{}({from})[{}]",
            RuntimeGlobals::REQUIRE,
            json_stringify(ident)
          )
        }
      })
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    writeln!(code, "  {}: {},", json_stringify(key), content).map_err(|e| error!(e.to_string()))?;
  }
  code += "}";
  code += right;
  code += ";\n";
  Ok(code)
}

pub fn css_modules_exports_to_concatenate_module_string(
  exports: &CssExports,
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
      .map(|CssExport { ident, from }| match from {
        None => json_stringify(ident),
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
          format!(
            "{}({from})[{}]",
            RuntimeGlobals::REQUIRE,
            json_stringify(ident)
          )
        }
      })
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    let mut identifier = to_identifier(&key);
    let mut i = 0;
    while used_identifiers.contains(&identifier) {
      identifier = format!("{key}{i}");
      i += 1;
    }
    // TODO: conditional support `const or var` after we finished runtimeTemplate utils
    concate_source.add(RawSource::from(format!("var {identifier} = {content};\n")));
    used_identifiers.insert(identifier.clone());
    scope.register_export(key.as_str().into(), identifier);
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

pub fn css_parsing_traceable_warning(
  source_code: impl Into<String>,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
  message: impl Into<String>,
) -> TraceableError {
  TraceableError::from_file(
    source_code.into(),
    start as usize,
    end as usize,
    "CSS parsing warning".to_string(),
    message.into(),
  )
}

pub fn replace_module_request_prefix(
  specifier: &str,
  diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  source_code: &str,
  start: css_module_lexer::Pos,
) -> String {
  if specifier.starts_with('~') {
    diagnostics.push(
      css_parsing_traceable_warning(
        source_code,
        start,
        start + 1,
        "'@import' or 'url()' with a request starts with '~' is deprecated.".to_string(),
      )
      .with_help(Some("Remove '~' from the request."))
      .with_severity(Severity::Warning)
      .boxed(),
    );
    String::from(&specifier[1..])
  } else {
    specifier.to_string()
  }
}
