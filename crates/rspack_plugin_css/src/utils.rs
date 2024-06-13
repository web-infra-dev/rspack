use std::borrow::Cow;
use std::fmt::Write;
use std::hash::Hasher;

use heck::{ToKebabCase, ToLowerCamelCase};
use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rspack_core::rspack_sources::{ConcatSource, RawSource};
use rspack_core::{
  to_identifier, Compilation, CompilerOptions, GenerateContext, PathData, ResourceData,
  RuntimeGlobals,
};
use rspack_core::{CssExportsConvention, LocalIdentName};
use rspack_error::{error, miette::Diagnostic, Result, TraceableError};
use rspack_error::{DiagnosticExt, RspackSeverity};
use rspack_hash::RspackHash;
use rspack_util::identifier::make_paths_relative;
use rspack_util::infallible::ResultInfallibleExt;
use rspack_util::json_stringify;
use rustc_hash::FxHashSet as HashSet;

use crate::parser_and_generator::CssExport;

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
pub static AUTO_PUBLIC_PATH_PLACEHOLDER_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(AUTO_PUBLIC_PATH_PLACEHOLDER).expect("Invalid regexp"));
pub static LEADING_DIGIT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\d+").expect("Invalid regexp"));

#[derive(Debug, Clone)]
pub struct LocalIdentOptions<'a> {
  relative_resource: String,
  local_name_ident: &'a LocalIdentName,
  compiler_options: &'a CompilerOptions,
}

impl<'a> LocalIdentOptions<'a> {
  pub fn new(
    resource_data: &ResourceData,
    local_name_ident: &'a LocalIdentName,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    let relative_resource = make_paths_relative(&compiler_options.context, &resource_data.resource);
    Self {
      relative_resource,
      local_name_ident,
      compiler_options,
    }
  }

  pub fn get_local_ident(&self, local: &str) -> String {
    let output = &self.compiler_options.output;
    let hash = {
      let mut hasher = RspackHash::with_salt(&output.hash_function, &output.hash_salt);
      hasher.write(self.relative_resource.as_bytes());
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
        .filename(&self.relative_resource)
        .hash(&hash)
        // TODO: should be moduleId, but we don't have it at parse,
        // and it's lots of work to move css module compile to generator,
        // so for now let's use hash for compatibility.
        .id(&PathData::prepare_id(
          if self.compiler_options.mode.is_development() {
            &self.relative_resource
          } else {
            &hash
          },
        )),
      local,
      unique_name: &output.unique_name,
    }
    .render_local_ident_name(self.local_name_ident)
  }
}

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
    s = s.replace("[uniqueName]", self.unique_name);
    s = escape_css_ident(&s);
    s = s.replace(r"\[local\]", self.local); // [local] is escaped to \[local\]
    s
  }
}

static UNESCAPE_CSS_IDENT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"([^a-zA-Z0-9_\u0081-\uffff-])").expect("invalid regex"));

pub fn escape_css_ident(s: &str) -> String {
  UNESCAPE_CSS_IDENT_REGEX.replace_all(s, "\\$1").into_owned()
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

pub fn css_modules_exports_to_string<'a>(
  exports: IndexMap<&'a str, &'a IndexSet<CssExport>>,
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
        None => json_stringify(&unescape(ident)),
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
            json_stringify(&unescape(ident))
          )
        }
      })
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    writeln!(code, "  {}: {},", json_stringify(&unescape(key)), content)
      .map_err(|e| error!(e.to_string()))?;
  }
  code += "}";
  code += right;
  code += ";\n";
  Ok(code)
}

pub fn css_modules_exports_to_concatenate_module_string<'a>(
  exports: IndexMap<&'a str, &'a IndexSet<CssExport>>,
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
        None => json_stringify(&unescape(ident)),
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
            json_stringify(&unescape(ident))
          )
        }
      })
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    let mut identifier = to_identifier(key);
    let mut i = 0;
    while used_identifiers.contains(&identifier) {
      identifier = format!("{key}{i}");
      i += 1;
    }
    // TODO: conditional support `const or var` after we finished runtimeTemplate utils
    concate_source.add(RawSource::from(format!("var {identifier} = {content};\n")));
    used_identifiers.insert(identifier.clone());
    scope.register_export(unescape(key).as_ref().into(), identifier);
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

pub fn unescape(s: &str) -> Cow<str> {
  UNESCAPE.replace_all(s.as_ref(), |caps: &Captures| {
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
  })
}

static WHITE_OR_BRACKET_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[\n\t ()'"\\]"#).expect("Invalid Regexp"));
static QUOTATION_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[\n"\\]"#).expect("Invalid Regexp"));
static APOSTROPHE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[\n'\\]"#).expect("Invalid Regexp"));

pub fn css_escape_string(s: &str) -> String {
  let mut count_white_or_bracket = 0;
  let mut count_quotation = 0;
  let mut count_apostrophe = 0;
  for c in s.chars() {
    match c {
      '\t' | '\n' | ' ' | '(' | ')' => count_white_or_bracket += 1,
      '"' => count_quotation += 1,
      '\'' => count_apostrophe += 1,
      _ => {}
    }
  }
  if count_white_or_bracket < 2 {
    WHITE_OR_BRACKET_REGEX
      .replace_all(s, |caps: &Captures| format!("\\{}", &caps[0]))
      .into_owned()
  } else if count_quotation <= count_apostrophe {
    format!(
      "\"{}\"",
      QUOTATION_REGEX.replace_all(s, |caps: &Captures| format!("\\{}", &caps[0]))
    )
  } else {
    format!(
      "\'{}\'",
      APOSTROPHE_REGEX.replace_all(s, |caps: &Captures| format!("\\{}", &caps[0]))
    )
  }
}

pub fn normalize_url(s: &str) -> String {
  let result = STRING_MULTILINE.replace_all(s, "");
  let result = TRIM_WHITE_SPACES.replace_all(&result, "");
  let result = unescape(&result);

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

pub fn css_parsing_traceable_error(
  source_code: impl Into<String>,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
  message: impl Into<String>,
  severity: RspackSeverity,
) -> TraceableError {
  TraceableError::from_file(
    source_code.into(),
    start as usize,
    end as usize,
    match severity {
      RspackSeverity::Error => "CSS parsing error".to_string(),
      RspackSeverity::Warn => "CSS parsing warning".to_string(),
    },
    message.into(),
  )
  .with_severity(severity)
}

pub fn replace_module_request_prefix<'s>(
  specifier: &'s str,
  diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  source_code: &str,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
) -> &'s str {
  if let Some(specifier) = specifier.strip_prefix('~') {
    diagnostics.push(
      css_parsing_traceable_error(
        source_code,
        start,
        end,
        "'@import' or 'url()' with a request starts with '~' is deprecated.".to_string(),
        RspackSeverity::Warn,
      )
      .with_help(Some("Remove '~' from the request."))
      .boxed(),
    );
    specifier
  } else {
    specifier
  }
}
