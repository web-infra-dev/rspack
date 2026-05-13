use std::{
  borrow::Cow,
  hash::Hasher,
  path::Path,
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use heck::{ToKebabCase, ToLowerCamelCase};
use regex::{Captures, Regex};
use rspack_core::{CompilerOptions, CssExportsConvention, LocalIdentName, PathData, ResourceData};
use rspack_error::{Diagnostic, Error, Result, Severity};
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash};
use rspack_util::identifier::make_paths_relative;

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
pub static LEADING_DIGIT_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^((-?[0-9])|--)").expect("Invalid regexp"));
static CSS_PREPARE_ID_LEADING_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^([.-]|[^a-zA-Z0-9_-])+").expect("Invalid regexp"));
static CSS_PREPARE_ID_REST_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9@_-]+").expect("Invalid regexp"));

fn css_prepare_id(v: &str) -> String {
  let without_leading = CSS_PREPARE_ID_LEADING_REGEX.replace(v, "");
  CSS_PREPARE_ID_REST_REGEX
    .replace_all(&without_leading, "_")
    .into_owned()
}

#[derive(Debug, Clone)]
pub struct LocalIdentOptions<'a> {
  relative_resource: String,
  local_name_ident: &'a LocalIdentName,
  compiler_options: &'a CompilerOptions,
  local_ident_hash_digest: Option<&'a HashDigest>,
  local_ident_hash_digest_length: Option<usize>,
  local_ident_hash_function: Option<&'a HashFunction>,
  local_ident_hash_salt: Option<&'a HashSalt>,
}

impl<'a> LocalIdentOptions<'a> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    resource_data: &ResourceData,
    local_name_ident: &'a LocalIdentName,
    compiler_options: &'a CompilerOptions,
    local_ident_hash_digest: Option<&'a HashDigest>,
    local_ident_hash_digest_length: Option<usize>,
    local_ident_hash_function: Option<&'a HashFunction>,
    local_ident_hash_salt: Option<&'a HashSalt>,
  ) -> Self {
    let relative_resource =
      make_paths_relative(&compiler_options.context, resource_data.resource());
    Self {
      relative_resource,
      local_name_ident,
      compiler_options,
      local_ident_hash_digest,
      local_ident_hash_digest_length,
      local_ident_hash_function,
      local_ident_hash_salt,
    }
  }

  pub async fn get_local_ident(&self, local: &str) -> Result<String> {
    let output = &self.compiler_options.output;
    let template = self
      .local_name_ident
      .template
      .template()
      .unwrap_or_default();
    let hash_function = self
      .local_ident_hash_function
      .unwrap_or(&output.hash_function);
    let hash_salt = self.local_ident_hash_salt.unwrap_or(&output.hash_salt);
    let hash_digest = self.local_ident_hash_digest.unwrap_or(&output.hash_digest);
    let hash_digest_length = self
      .local_ident_hash_digest_length
      .unwrap_or(output.hash_digest_length);
    let hash = if template.contains("[hash") || template.contains("[fullhash") {
      let mut hasher = RspackHash::with_salt(hash_function, hash_salt);
      if !output.unique_name.is_empty() {
        hasher.write(output.unique_name.as_bytes());
      }
      hasher.write(self.relative_resource.as_bytes());
      hasher.write(local.as_bytes());
      hasher
        .digest(hash_digest)
        .rendered(hash_digest_length)
        .to_string()
    } else {
      String::new()
    };
    let resource_path = self
      .relative_resource
      .split(['?', '#'])
      .next()
      .unwrap_or(&self.relative_resource);
    let ext = Path::new(resource_path)
      .extension()
      .and_then(|s| s.to_str())
      .map(|ext| format!(".{ext}"))
      .unwrap_or_default();
    let chunk_name = Path::new(resource_path)
      .file_name()
      .and_then(|s| s.to_str())
      .map(|base| base.strip_suffix(&ext).unwrap_or(base).to_string())
      .unwrap_or_default();
    let id = css_prepare_id(if self.compiler_options.mode.is_development() {
      &self.relative_resource
    } else {
      &hash
    });
    let local_ident = LocalIdentNameRenderOptions {
      path_data: PathData::default()
        .filename(&self.relative_resource)
        .chunk_name(&chunk_name)
        .hash(&hash)
        .id(&id),
      local,
      unique_name: &output.unique_name,
      folder: Path::new(&self.relative_resource)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or(""),
    }
    .render_local_ident_name(self.local_name_ident)
    .await?;
    Ok(
      LEADING_DIGIT_REGEX
        .replace(&local_ident, "_${1}")
        .into_owned(),
    )
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
