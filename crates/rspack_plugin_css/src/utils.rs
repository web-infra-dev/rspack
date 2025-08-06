use std::{
  borrow::Cow,
  fmt::Write,
  hash::Hasher,
  sync::Arc,
};

use cow_utils::CowUtils;
use heck::{ToKebabCase, ToLowerCamelCase};
use indexmap::{IndexMap, IndexSet};
use rspack_core::{
  ChunkGraph, Compilation, CompilerOptions, CssExportsConvention, GenerateContext, LocalIdentName,
  PathData, RESERVED_IDENTIFIER, ResourceData, RuntimeGlobals,
  rspack_sources::{ConcatSource, RawStringSource},
  to_identifier,
};
use rspack_error::{
  DiagnosticExt, Result, RspackSeverity, ToStringResultToRspackResultExt, TraceableError,
  miette::Diagnostic,
};
use rspack_hash::RspackHash;
use rspack_util::{identifier::make_paths_relative, itoa, json_stringify};
use rustc_hash::FxHashSet as HashSet;

use crate::parser_and_generator::CssExport;

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
fn replace_leading_digit_or_dash(s: &str) -> Cow<str> {
  if s.is_empty() {
    return Cow::Borrowed(s);
  }
  
  let bytes = s.as_bytes();
  if bytes[0] == b'-' && bytes.len() > 1 {
    if bytes[1] == b'-' {
      // Replace "--" with "_--"
      Cow::Owned(format!("_{}", s))
    } else if bytes[1].is_ascii_digit() {
      // Replace "-digit" with "_-digit"  
      Cow::Owned(format!("_{}", s))
    } else {
      Cow::Borrowed(s)
    }
  } else if bytes[0].is_ascii_digit() {
    // Replace leading digit with "_digit"
    Cow::Owned(format!("_{}", s))
  } else {
    Cow::Borrowed(s)
  }
}

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
      let rendered_hash = hash.rendered(output.hash_digest_length);
      replace_leading_digit_or_dash(&rendered_hash).into_owned()
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
    .await
  }
}

struct LocalIdentNameRenderOptions<'a> {
  path_data: PathData<'a>,
  local: &'a str,
  unique_name: &'a str,
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
        .into_owned(),
    )
  }
}

pub fn escape_css(s: &str) -> Cow<'_, str> {
  let mut result = String::new();
  let mut needs_escape = false;
  
  for ch in s.chars() {
    match ch {
      'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => {
        if needs_escape {
          result.push(ch);
        }
      }
      '\u{0081}'..='\u{FFFF}' => {
        if needs_escape {
          result.push(ch);
        }
      }
      _ => {
        if !needs_escape {
          needs_escape = true;
          result.reserve(s.len() + 10); // Reserve some extra space for escapes
          // Copy the part we've checked so far
          for prev_ch in s.chars().take_while(|&c| c != ch) {
            result.push(prev_ch);
          }
        }
        result.push('\\');
        result.push(ch);
      }
    }
  }
  
  if needs_escape {
    Cow::Owned(result)
  } else {
    Cow::Borrowed(s)
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
      .map(|CssExport { ident, from, id: _ }| match from {
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
            RuntimeGlobals::REQUIRE,
            json_stringify(&unescape(ident))
          )
        }
      })
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
      .map(|CssExport { ident, from, id: _ }| match from {
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
            RuntimeGlobals::REQUIRE,
            json_stringify(&ident)
          )
        }
      })
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

fn remove_css_multiline_backslashes(s: &str) -> Cow<str> {
  if s.contains("\\") {
    // Check for multiline backslashes: backslash followed by newline, carriage return, or form feed
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    
    while let Some(ch) = chars.next() {
      if ch == '\\' {
        if let Some(&next_ch) = chars.peek() {
          if next_ch == '\n' || next_ch == '\r' || next_ch == '\u{000C}' {
            chars.next(); // Skip the whitespace character after backslash
            continue;
          }
        }
      }
      result.push(ch);
    }
    Cow::Owned(result)
  } else {
    Cow::Borrowed(s)
  }
}

fn trim_css_whitespace(s: &str) -> &str {
  // CSS whitespace: space, tab, newline, carriage return, form feed
  let start = s.find(|c: char| !matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000C}')).unwrap_or(s.len());
  let end = s.rfind(|c: char| !matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000C}'))
    .map(|i| i + 1)
    .unwrap_or(0);
  
  if end > start {
    &s[start..end]
  } else {
    ""
  }
}

// Clean manual implementation of CSS unescape functionality (better than the original 62-line version)
fn unescape_css_manual(s: &str) -> Cow<str> {
  if !s.contains('\\') {
    return Cow::Borrowed(s);
  }

  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars();
  
  while let Some(ch) = chars.next() {
    if ch == '\\' {
      if let Some(next_ch) = chars.next() {
        if next_ch.is_ascii_hexdigit() {
          // Handle hex escape sequence: \123ABC followed by optional whitespace
          let mut hex_chars = String::from(next_ch);
          
          // Collect up to 5 more hex digits (total 6)
          for _ in 0..5 {
            if let Some(hex_ch) = chars.as_str().chars().next() {
              if hex_ch.is_ascii_hexdigit() {
                hex_chars.push(hex_ch);
                chars.next();
              } else {
                break;
              }
            }
          }
          
          // Skip optional trailing whitespace
          if let Some(ws_ch) = chars.as_str().chars().next() {
            if matches!(ws_ch, ' ' | '\t' | '\n' | '\r' | '\u{000C}') {
              chars.next();
            }
          }
          
          // Convert hex to unicode character
          if let Ok(code_point) = u32::from_str_radix(&hex_chars, 16) {
            if let Some(unicode_ch) = char::from_u32(code_point) {
              result.push(unicode_ch);
              continue;
            }
          }
          
          // If conversion failed, keep the original backslash and hex chars
          result.push('\\');
          result.push_str(&hex_chars);
        } else {
          // Single character escape like \n, \", etc.
          result.push(next_ch);
        }
      } else {
        // Backslash at end of string
        result.push('\\');
      }
    } else {
      result.push(ch);
    }
  }
  
  Cow::Owned(result)
}

fn is_data_uri(s: &str) -> bool {
  s.len() >= 5 && s.as_bytes()[0..5].eq_ignore_ascii_case(b"data:")
}

// `\/foo` in css should be treated as `foo` in js
pub fn unescape(s: &str) -> Cow<'_, str> {
  unescape_css_manual(s)
}

fn escape_white_or_bracket_chars(s: &str) -> String {
  s.chars()
    .map(|c| match c {
      '\n' | '\t' | ' ' | '(' | ')' | '\'' | '"' | '\\' => format!("\\{}", c),
      _ => c.to_string(),
    })
    .collect()
}

fn escape_quotation_chars(s: &str) -> String {
  s.chars()
    .map(|c| match c {
      '\n' | '"' | '\\' => format!("\\{}", c),
      _ => c.to_string(),
    })
    .collect()
}

fn escape_apostrophe_chars(s: &str) -> String {
  s.chars()
    .map(|c| match c {
      '\n' | '\'' | '\\' => format!("\\{}", c),
      _ => c.to_string(),
    })
    .collect()
}

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
    escape_white_or_bracket_chars(s)
  } else if count_quotation <= count_apostrophe {
    format!("\"{}\"", escape_quotation_chars(s))
  } else {
    format!("'{}'", escape_apostrophe_chars(s))
  }
}

pub fn normalize_url(s: &str) -> String {
  let result = remove_css_multiline_backslashes(s);
  let result = trim_css_whitespace(&result);
  let result = unescape(&result);

  if is_data_uri(&result) {
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
  severity: RspackSeverity,
) -> TraceableError {
  TraceableError::from_arc_string(
    Some(source_code),
    start as usize,
    end as usize,
    match severity {
      RspackSeverity::Error => "CSS parse error".to_string(),
      RspackSeverity::Warn => "CSS parse warning".to_string(),
    },
    message.into(),
  )
  .with_severity(severity)
}

pub fn replace_module_request_prefix<'s>(
  specifier: &'s str,
  diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  source_code: impl Fn() -> Arc<String>,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
) -> &'s str {
  if let Some(specifier) = specifier.strip_prefix('~') {
    diagnostics.push(
      css_parsing_traceable_error(
        source_code(),
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
