use std::{
  borrow::Cow,
  hash::Hasher,
  path::Path,
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use heck::{ToKebabCase, ToLowerCamelCase};
use once_cell::sync::OnceCell;
use regex::{Captures, Regex};
use rspack_core::{
  ChunkGraph, Compilation, CompilerOptions, CssExportsConvention, CssModuleGeneratorOptions,
  LocalIdentName, Module, ModuleType, PathData, ReplaceAllPlaceholder, ResourceData,
};
use rspack_error::{Diagnostic, Error, Result, Severity};
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash};
use rspack_util::{base64, identifier::make_paths_relative, itoa, json_stringify_str};
use rustc_hash::FxHashSet;

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";
pub const CSS_MODULE_ID_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_MODULE_ID__";
pub static LEADING_DIGIT_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^((-?[0-9])|--)").expect("Invalid regexp"));

#[derive(Debug, Clone)]
pub struct PresentationalDependencyHashUpdate<'a> {
  pub start: u32,
  pub end: u32,
  pub content: &'a str,
}

#[derive(Debug, Clone)]
pub struct LocalIdentModuleHashOptions<'a> {
  pub export_dependency_names: Vec<String>,
  pub graph_export_names: FxHashSet<String>,
  pub presentational_dependency_hash_updates: Vec<PresentationalDependencyHashUpdate<'a>>,
  pub exports_only: bool,
  pub es_module: bool,
  pub named_exports: bool,
  pub exports_convention: Option<CssExportsConvention>,
}

#[derive(Debug, Clone)]
pub struct LocalIdentOptions<'a> {
  relative_resource: String,
  module_type: &'static str,
  source: Arc<str>,
  module_hash: OnceCell<String>,
  compiler_options: &'a CompilerOptions,
  local_ident_name: &'a LocalIdentName,
  local_ident_hash_digest: HashDigest,
  local_ident_hash_digest_length: usize,
  local_ident_hash_function: HashFunction,
  local_ident_hash_salt: &'a HashSalt,
}

impl<'a> LocalIdentOptions<'a> {
  pub fn new(
    resource_data: &ResourceData,
    module_type: &ModuleType,
    source: Arc<str>,
    compiler_options: &'a CompilerOptions,
    generator_options: &'a CssModuleGeneratorOptions,
  ) -> Self {
    let relative_resource =
      make_paths_relative(&compiler_options.context, resource_data.resource());

    let local_ident_name = generator_options
      .local_ident_name
      .as_ref()
      .expect("should have local_ident_name when calculating css local ident module hash");
    let local_ident_hash_digest = generator_options
      .local_ident_hash_digest
      .expect("should have local_ident_hash_digest when calculating css local ident module hash");
    let local_ident_hash_digest_length = generator_options
      .local_ident_hash_digest_length
      .map(|len| len as usize)
      .expect(
        "should have local_ident_hash_digest_length when calculating css local ident module hash",
      );
    let local_ident_hash_function = generator_options
      .local_ident_hash_function
      .expect("should have local_ident_hash_function when calculating css local ident module hash");
    let local_ident_hash_salt = &generator_options.local_ident_hash_salt;

    Self {
      relative_resource,
      module_type: module_type.as_str(),
      source,
      module_hash: OnceCell::new(),
      compiler_options,
      local_ident_name,
      local_ident_hash_digest,
      local_ident_hash_digest_length,
      local_ident_hash_function,
      local_ident_hash_salt,
    }
  }

  fn module_hash(&self, module_hash_options: &LocalIdentModuleHashOptions<'_>) -> &str {
    self
      .module_hash
      .get_or_init(|| self.get_module_hash(module_hash_options))
      .as_str()
  }

  fn get_module_hash(&self, module_hash_options: &LocalIdentModuleHashOptions<'_>) -> String {
    let local_ident_name = self.local_ident_name.template.as_str();
    let build_hash = {
      let mut hasher = RspackHash::new(&self.local_ident_hash_function);
      hasher.write(b"source");
      hasher.write(b"OriginalSource");
      hasher.write(self.source.as_bytes());
      hasher.write(format!("webpack://{}|{}", self.module_type, self.relative_resource).as_bytes());
      hasher.write(b"meta");
      if module_hash_options.named_exports {
        hasher.write(br#"{"isCSSModule":true,"exportsType":"namespace","defaultObject":false}"#);
      } else {
        hasher.write(
          br#"{"isCSSModule":true,"exportsType":"default","defaultObject":"redirect-warn"}"#,
        );
      }
      hasher.digest(&HashDigest::Hex).encoded().to_string()
    };

    let graph_hash = {
      let mut graph_exports = module_hash_options
        .graph_export_names
        .iter()
        .collect::<Vec<_>>();
      graph_exports.sort();

      let mut hasher = RspackHash::new(&self.local_ident_hash_function);
      hasher.write(self.relative_resource.as_bytes());
      hasher.write(b"false");
      for name in graph_exports {
        hasher.write(name.as_bytes());
        hasher.write(b"2truefalse");
      }
      hasher.write(b"*side effects only*2undefinedfalse");
      hasher.write(b"null2falsefalse");
      hasher.digest(&HashDigest::Hex).encoded().to_string()
    };

    let mut hasher =
      RspackHash::with_salt(&self.local_ident_hash_function, self.local_ident_hash_salt);
    hasher.write(build_hash.as_bytes());
    if module_hash_options.exports_only {
      hasher.write(b"javascript");
    } else {
      hasher.write(b"javascript");
      hasher.write(b"css");
    }
    hasher.write(if module_hash_options.es_module {
      b"true"
    } else {
      b"false"
    });
    hasher.write(if module_hash_options.exports_only {
      b"true"
    } else {
      b"false"
    });
    hasher.write(graph_hash.as_bytes());
    let mut itoa_buffer = itoa::Buffer::new();
    for update in module_hash_options
      .presentational_dependency_hash_updates
      .iter()
    {
      hasher.write(itoa_buffer.format(update.start).as_bytes());
      hasher.write(b",");
      hasher.write(itoa_buffer.format(update.end).as_bytes());
      hasher.write(b"|");
      hasher.write(update.content.as_bytes());
    }
    for name in module_hash_options.export_dependency_names.iter() {
      let convention_names = export_locals_convention(
        name,
        module_hash_options
          .exports_convention
          .expect("should have convention for module_type css/auto, css/global or css/module"),
      );
      let convention_names =
        serde_json::to_string(&convention_names).expect("css export names should be serializable");
      let local_ident_name = json_stringify_str(local_ident_name);
      hasher.write(b"exportsConvention|");
      hasher.write(convention_names.as_bytes());
      hasher.write(b"|localIdentName|");
      hasher.write(local_ident_name.as_bytes());
    }
    hasher
      .digest(&self.local_ident_hash_digest)
      .rendered(self.local_ident_hash_digest_length)
      .to_string()
  }

  pub async fn get_local_ident(
    &self,
    local: &str,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<String> {
    let output = &self.compiler_options.output;
    let local_ident_hash = {
      let mut hasher =
        RspackHash::with_salt(&self.local_ident_hash_function, self.local_ident_hash_salt);
      if !output.unique_name.is_empty() {
        hasher.write(output.unique_name.as_bytes());
      }
      hasher.write(self.relative_resource.as_bytes());
      hasher.write(local.as_bytes());
      let hash = hasher.digest(&self.local_ident_hash_digest);
      hash
        .rendered(self.local_ident_hash_digest_length)
        .to_string()
    };
    let content_hash;
    let content_hash = if self
      .local_ident_name
      .template
      .as_str()
      .contains("[contenthash")
    {
      let mut hasher = RspackHash::new(&output.hash_function);
      hasher.write(self.source.as_bytes());
      let hash = hasher.digest(&output.hash_digest);
      content_hash = non_numeric_only_hash(hash.encoded(), output.hash_digest_length);
      content_hash.as_str()
    } else {
      ""
    };
    let resource_path = self
      .relative_resource
      .split(['?', '#'])
      .next()
      .unwrap_or(&self.relative_resource);
    let resource_path = Path::new(resource_path);
    let chunk_name = resource_path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or_default();
    let id = PathData::prepare_id(CSS_MODULE_ID_PLACEHOLDER);
    let local_ident = LocalIdentNameRenderOptions {
      path_data: PathData::default()
        .filename(&self.relative_resource)
        .chunk_name(chunk_name)
        .hash(self.module_hash(module_hash_options))
        .content_hash(content_hash)
        .id(id.as_ref()),
      local,
      local_ident_hash: &local_ident_hash,
      unique_name: &output.unique_name,
      folder: Path::new(&self.relative_resource)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or(""),
    }
    .render_local_ident_name(self.local_ident_name)
    .await?;
    Ok(
      LEADING_DIGIT_REGEX
        .replace(&local_ident, "_${1}")
        .into_owned(),
    )
  }
}

pub fn replace_css_module_id_placeholder<'a>(
  local_ident: &'a str,
  compilation: &Compilation,
  module: &dyn Module,
) -> Cow<'a, str> {
  if let Some(custom_property_ident) = local_ident.strip_prefix("--") {
    let local_ident = replace_css_module_id_placeholder(custom_property_ident, compilation, module);
    return Cow::Owned(format!("--{local_ident}"));
  }
  if !local_ident.contains(CSS_MODULE_ID_PLACEHOLDER) {
    return Cow::Borrowed(local_ident);
  }
  let module_id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
    .expect("css module should have module id when rendering local ident");
  let module_id = prepare_css_module_id(module_id.as_str());
  let local_ident = local_ident.cow_replace(CSS_MODULE_ID_PLACEHOLDER, module_id.as_ref());
  Cow::Owned(
    LEADING_DIGIT_REGEX
      .replace(&local_ident, "_${1}")
      .into_owned(),
  )
}

static PREPARE_CSS_MODULE_ID_START_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^([.-]|[^a-zA-Z0-9_-])+").expect("invalid Regex"));
static PREPARE_CSS_MODULE_ID_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9@_-]+").expect("invalid Regex"));

fn prepare_css_module_id(v: &str) -> Cow<'_, str> {
  let v = PREPARE_CSS_MODULE_ID_START_REGEX.replace(v, "");
  Cow::Owned(
    PREPARE_CSS_MODULE_ID_REGEX
      .replace_all(&v, "_")
      .into_owned(),
  )
}

struct LocalIdentNameRenderOptions<'a> {
  path_data: PathData<'a>,
  local: &'a str,
  local_ident_hash: &'a str,
  unique_name: &'a str,
  folder: &'a str,
}

fn render_hash(hash: &str, len: Option<usize>, need_base64: bool) -> String {
  let content = if need_base64 {
    base64::encode_to_string(hash)
  } else {
    hash.to_string()
  };
  content[..len.unwrap_or(content.len()).min(content.len())].to_string()
}

fn non_numeric_only_hash(hash: &str, hash_length: usize) -> String {
  if hash_length < 1 {
    return String::new();
  }
  let len = hash_length.min(hash.len());
  let slice = &hash[..len];
  if slice.bytes().any(|b| !b.is_ascii_digit()) {
    return slice.to_string();
  }
  let first = hash
    .as_bytes()
    .first()
    .copied()
    .filter(u8::is_ascii_digit)
    .map_or(0, |b| b - b'0');
  format!("{}{}", char::from(b'a' + (first % 6)), &slice[1..])
}

impl LocalIdentNameRenderOptions<'_> {
  pub async fn render_local_ident_name(self, local_ident_name: &LocalIdentName) -> Result<String> {
    let template = local_ident_name.template.template().map_or_else(
      || local_ident_name.template.clone(),
      |template| {
        template
          .replace_all_with_len("[fullhash]", |len, need_base64| {
            render_hash(self.local_ident_hash, len, need_base64)
          })
          .into_owned()
          .into()
      },
    );
    let raw = template.render(self.path_data, None).await?;
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
  locals_convention: CssExportsConvention,
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

pub fn css_parsing_traceable_error(
  source_code: &str,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
  message: impl Into<String>,
  severity: Severity,
) -> Error {
  let mut error = Error::from_string(
    Some(source_code.to_owned()),
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
  source_code: &str,
  start: css_module_lexer::Pos,
  end: css_module_lexer::Pos,
) -> &'s str {
  if let Some(specifier) = specifier.strip_prefix('~') {
    let mut error = css_parsing_traceable_error(
      source_code,
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
