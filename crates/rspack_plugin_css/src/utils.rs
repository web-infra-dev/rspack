use std::{
  borrow::Cow,
  fmt::Write,
  hash::Hasher,
  path::Path,
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use heck::{ToKebabCase, ToLowerCamelCase};
use indexmap::{IndexMap, IndexSet};
use regex::{Captures, Regex};
use rspack_core::{
  ChunkGraph, Compilation, CompilerOptions, CssExportsConvention, GenerateContext, LocalIdentName,
  PathData, RESERVED_IDENTIFIER, ResourceData, RuntimeGlobals,
  rspack_sources::{ConcatSource, RawStringSource},
  to_identifier,
};
use rspack_error::{Diagnostic, Error, Result, Severity, ToStringResultToRspackResultExt};
use rspack_hash::RspackHash;
use rspack_util::{identifier::make_paths_relative, itoa, json_stringify};
use rustc_hash::FxHashSet as HashSet;

use crate::parser_and_generator::CssExport;

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
pub static LEADING_DIGIT_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^((-?[0-9])|--)").expect("Invalid regexp"));

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
    let relative_resource =
      make_paths_relative(&compiler_options.context, resource_data.resource());
    Self {
      relative_resource,
      local_name_ident,
      compiler_options,
    }
  }

  pub async fn get_local_ident(&self, local: &str) -> Result<String> {
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
        .replace(hash.rendered(output.hash_digest_length), "_${1}")
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
      folder: Path::new(&self.relative_resource)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or(""),
    }
    .render_local_ident_name(self.local_name_ident)
    .await
  }
}

struct LocalIdentNameRenderOptions<'a> {
  path_data: PathData<'a>,
  local: &'a str,
  unique_name: &'a str,
  folder: &'a str,
}

impl LocalIdentNameRenderOptions<'_> {
  pub async fn render_local_ident_name(self, local_ident_name: &LocalIdentName) -> Result<String> {
    let raw = local_ident_name
      .template
      .render(self.path_data, None)
      .await?;
    let s: &str = raw.as_ref();

    Ok(
      s.cow_replace("[uniqueName]", self.unique_name)
        .cow_replace("[local]", self.local)
        .cow_replace("[folder]", self.folder)
        .into_owned(),
    )
  }
}

static UNESCAPE_CSS_IDENT_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"([^a-zA-Z0-9_\u0081-\uffff-])").expect("invalid regex"));

pub fn escape_css(s: &str) -> Cow<'_, str> {
  UNESCAPE_CSS_IDENT_REGEX.replace_all(s, |s: &Captures| format!("\\{}", &s[0]))
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

#[allow(clippy::too_many_arguments)]
pub fn css_modules_exports_to_string<'a>(
  exports: IndexMap<&'a str, &'a IndexSet<CssExport>>,
  module: &dyn rspack_core::Module,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  ns_obj: &str,
  left: &str,
  right: &str,
  with_hmr: bool,
) -> Result<String> {
  let (decl_name, exports_string) =
    stringified_exports(exports, compilation, runtime_requirements, module)?;

  let hmr_code = if with_hmr {
    Cow::Owned(format!(
      "// only invalidate when locals change
var stringified_exports = JSON.stringify({decl_name});
if (module.hot.data && module.hot.data.exports && module.hot.data.exports != stringified_exports) {{
  module.hot.invalidate();
}} else {{
  module.hot.accept(); 
}}
module.hot.dispose(function(data) {{ data.exports = stringified_exports; }});"
    ))
  } else {
    Cow::Borrowed("")
  };
  let mut code =
    format!("{exports_string}\n{hmr_code}\n{ns_obj}{left}module.exports = {decl_name}",);
  code += right;
  code += ";\n";
  Ok(code)
}

pub fn stringified_exports<'a>(
  exports: IndexMap<&'a str, &'a IndexSet<CssExport>>,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  module: &dyn rspack_core::Module,
) -> Result<(&'static str, String)> {
  let mut stringified_exports = String::new();
  let module_graph = compilation.get_module_graph();
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(
        |CssExport {
           ident,
           from,
           id: _,
           orig_name: _,
         }| match from {
          None => json_stringify(&ident),
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

            let from = serde_json::to_string(
              ChunkGraph::get_module_id(&compilation.module_ids_artifact, from.module_identifier)
                .expect("should have module"),
            )
            .expect("should json stringify module id");
            runtime_requirements.insert(RuntimeGlobals::REQUIRE);
            format!(
              "{}({from})[{}]",
              compilation
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::REQUIRE),
              json_stringify(&unescape(ident))
            )
          }
        },
      )
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    writeln!(
      stringified_exports,
      "  {}: {},",
      json_stringify(&key),
      content
    )
    .to_rspack_result()?;
  }

  let decl_name = "exports";
  Ok((
    decl_name,
    format!("var {decl_name} = {{\n{stringified_exports}}};"),
  ))
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
  let Some(scope) = concatenation_scope else {
    return Ok(());
  };
  let module_graph = compilation.get_module_graph();
  let mut used_identifiers = HashSet::default();
  for (key, elements) in exports {
    let content = elements
      .iter()
      .map(
        |CssExport {
           ident,
           from,
           id: _,
           orig_name: _,
         }| match from {
          None => json_stringify(&ident),
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

            let from = serde_json::to_string(
              ChunkGraph::get_module_id(&compilation.module_ids_artifact, from.module_identifier)
                .expect("should have module"),
            )
            .expect("should json stringify module id");
            format!(
              "{}({from})[{}]",
              compilation
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::REQUIRE),
              json_stringify(&ident)
            )
          }
        },
      )
      .collect::<Vec<_>>()
      .join(" + \" \" + ");
    let mut identifier = to_identifier(key);
    if RESERVED_IDENTIFIER.contains(identifier.as_ref()) {
      identifier = Cow::Owned(format!("_{identifier}"));
    }
    let mut i = 0;
    while used_identifiers.contains(&identifier) {
      let mut i_buffer = itoa::Buffer::new();
      let i_str = i_buffer.format(i);
      identifier = Cow::Owned(format!("{identifier}{}", i_str));
      i += 1;
    }
    // TODO: conditional support `const or var` after we finished runtimeTemplate utils
    concate_source.add(RawStringSource::from(format!(
      "var {identifier} = {content};\n"
    )));
    used_identifiers.insert(identifier.clone());
    scope.register_export(key.into(), identifier.into_owned());
  }
  Ok(())
}

static STRING_MULTILINE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\\[\n\r\f]").expect("Invalid RegExp"));

static TRIM_WHITE_SPACES: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(^[ \t\n\r\f]*|[ \t\n\r\f]*$)").expect("Invalid RegExp"));

static UNESCAPE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\\([0-9a-fA-F]{1,6}[ \t\n\r\f]?|[\s\S])").expect("Invalid RegExp"));

static DATA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?i)data:").expect("Invalid RegExp"));

// `\/foo` in css should be treated as `foo` in js
pub fn unescape(s: &str) -> Cow<'_, str> {
  UNESCAPE.replace_all(s.as_ref(), |caps: &Captures| {
    caps
      .get(0)
      .and_then(|m| {
        let m = m.as_str();
        if m.len() > 2 {
          if let Ok(r_u32) = u32::from_str_radix(m[1..].trim(), 16)
            && let Some(ch) = char::from_u32(r_u32)
          {
            return Some(format!("{ch}"));
          }
          None
        } else {
          Some(m[1..2].to_string())
        }
      })
      .unwrap_or(caps[0].to_string())
  })
}

static WHITE_OR_BRACKET_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"[\n\t ()'"\\]"#).expect("Invalid Regexp"));
static QUOTATION_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"[\n"\\]"#).expect("Invalid Regexp"));
static APOSTROPHE_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"[\n'\\]"#).expect("Invalid Regexp"));

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
  if result.contains('%')
    && let Ok(r) = urlencoding::decode(&result)
  {
    return r.to_string();
  }

  result.to_string()
}

#[allow(clippy::rc_buffer)]
pub fn css_parsing_traceable_error(
  source_code: Arc<String>,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
  message: impl Into<String>,
  severity: Severity,
) -> Error {
  let mut error = Error::from_string(
    Some(source_code.to_string()),
    start as usize,
    end as usize,
    match severity {
      Severity::Error => "CSS parse error".to_string(),
      Severity::Warning => "CSS parse warning".to_string(),
    },
    message.into(),
  );
  error.severity = severity;
  error
}

pub fn replace_module_request_prefix<'s>(
  specifier: &'s str,
  diagnostics: &mut Vec<Diagnostic>,
  source_code: impl Fn() -> Arc<String>,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
) -> &'s str {
  if let Some(specifier) = specifier.strip_prefix('~') {
    let mut error = css_parsing_traceable_error(
      source_code(),
      start,
      end,
      "'@import' or 'url()' with a request starts with '~' is deprecated.".to_string(),
      Severity::Warning,
    );
    error.help = Some("Remove '~' from the request.".into());
    diagnostics.push(error.into());
    specifier
  } else {
    specifier
  }
}
