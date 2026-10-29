//! Compiler options (less.js option names in snake_case).

use cow_utils::CowUtils;

/// How math operations are evaluated (`math` option).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Math {
  /// Everything is evaluated (`always`, deprecated).
  Always = 0,
  /// Division only inside parentheses (`parens-division`, the default).
  #[default]
  ParensDivision = 1,
  /// Only operations in parentheses are evaluated (`parens` / `strict`).
  Parens = 2,
}

impl Math {
  /// Parses the less.js spelling; unknown strings map to `Parens` like less.js.
  pub fn from_name(s: &str) -> Math {
    match s.cow_to_ascii_lowercase().as_ref() {
      "always" => Math::Always,
      "parens-division" => Math::ParensDivision,
      _ => Math::Parens,
    }
  }
}

/// URL rewriting mode (`rewriteUrls`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RewriteUrls {
  #[default]
  Off,
  Local,
  All,
}

impl RewriteUrls {
  pub fn from_name(s: &str) -> Option<RewriteUrls> {
    match s.cow_to_ascii_lowercase().as_ref() {
      "off" => Some(RewriteUrls::Off),
      "local" => Some(RewriteUrls::Local),
      "all" => Some(RewriteUrls::All),
      _ => None,
    }
  }
}

/// `dumpLineNumbers` (deprecated in less.js but still supported).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DumpLineNumbers {
  Comments,
  MediaQuery,
  All,
}

impl DumpLineNumbers {
  pub fn from_name(s: &str) -> Option<DumpLineNumbers> {
    match s {
      "comments" => Some(DumpLineNumbers::Comments),
      "mediaquery" => Some(DumpLineNumbers::MediaQuery),
      "all" => Some(DumpLineNumbers::All),
      _ => None,
    }
  }
}

/// Source map options (the `sourceMap` object of less.js).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceMapOptions {
  pub source_map_url: Option<String>,
  pub source_map_basepath: Option<String>,
  pub source_map_rootpath: Option<String>,
  pub source_map_filename: Option<String>,
  pub source_map_output_filename: Option<String>,
  pub source_map_input_filename: Option<String>,
  pub source_map_full_filename: Option<String>,
  pub source_map_file_inline: bool,
  pub output_source_files: bool,
  pub disable_sourcemap_annotation: bool,
}

impl SourceMapOptions {
  /// An external map written next to the CSS as `filename`.
  pub fn external(filename: impl Into<String>) -> SourceMapOptions {
    SourceMapOptions {
      source_map_filename: Some(filename.into()),
      ..Default::default()
    }
  }
  /// An inline (data-URI) map.
  pub fn inline() -> SourceMapOptions {
    SourceMapOptions {
      source_map_file_inline: true,
      ..Default::default()
    }
  }
  /// Embed the sources (`outputSourceFiles`).
  pub fn with_sources_content(mut self) -> SourceMapOptions {
    self.output_source_files = true;
    self
  }
}

/// All compiler options.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Options {
  /// Name of the input file (used for errors, relative imports, source maps).
  pub filename: Option<String>,
  pub compress: bool,
  pub strict_units: bool,
  pub math: Math,
  pub rewrite_urls: RewriteUrls,
  pub rootpath: String,
  pub url_args: String,
  pub include_paths: Vec<String>,
  pub strict_imports: bool,
  pub javascript_enabled: bool,
  pub dump_line_numbers: Option<DumpLineNumbers>,
  pub global_vars: Vec<(String, String)>,
  pub modify_vars: Vec<(String, String)>,
  pub banner: Option<String>,
  pub source_map: Option<SourceMapOptions>,
  pub process_imports: bool,
  pub quiet: bool,
  pub quiet_deprecations: bool,
  pub insecure: bool,
  pub disable_plugin_rule: bool,
}

impl Default for Options {
  fn default() -> Options {
    Options {
      filename: None,
      compress: false,
      strict_units: false,
      math: Math::ParensDivision,
      rewrite_urls: RewriteUrls::Off,
      rootpath: String::new(),
      url_args: String::new(),
      include_paths: Vec::new(),
      strict_imports: false,
      javascript_enabled: false,
      dump_line_numbers: None,
      global_vars: Vec::new(),
      modify_vars: Vec::new(),
      banner: None,
      source_map: None,
      process_imports: true,
      quiet: false,
      quiet_deprecations: false,
      insecure: false,
      disable_plugin_rule: false,
    }
  }
}

/// `Parser.serializeVars`.
pub(crate) fn serialize_vars(vars: &[(String, String)]) -> String {
  let mut s = String::new();
  for (name, value) in vars {
    if !name.starts_with('@') {
      s.push('@');
    }
    s.push_str(name);
    s.push_str(": ");
    s.push_str(value);
    if !value.ends_with(';') {
      s.push(';');
    }
  }
  s
}

#[cfg(feature = "json")]
impl Options {
  /// Builds options from a less.js options object (camelCase names, the
  /// same enum strings and legacy aliases such as `strictMath` and
  /// `relativeUrls`).
  pub fn from_json(v: &serde_json::Value) -> Options {
    use serde_json::Value as J;
    let mut o = Options::default();
    let obj = match v.as_object() {
      Some(m) => m,
      None => return o,
    };
    let s = |k: &str| obj.get(k).and_then(|x| x.as_str()).map(|x| x.to_string());
    let b = |k: &str| obj.get(k).and_then(|x| x.as_bool());
    let vars = |k: &str| -> Vec<(String, String)> {
      match obj.get(k) {
        Some(J::Object(m)) => m
          .iter()
          .map(|(k, v)| {
            (
              k.clone(),
              v.as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| v.to_string()),
            )
          })
          .collect(),
        _ => Vec::new(),
      }
    };
    o.filename = s("filename");
    o.compress = b("compress").unwrap_or(false);
    o.strict_units = b("strictUnits").unwrap_or(false);
    match obj.get("math") {
      Some(J::String(m)) => o.math = Math::from_name(m),
      Some(J::Number(n)) => {
        o.math = match n.as_u64() {
          Some(0) => Math::Always,
          Some(1) => Math::ParensDivision,
          _ => Math::Parens,
        }
      }
      _ => {}
    }
    // Less.js applies legacy aliases after copying the explicit options.
    if b("strictMath") == Some(true) {
      o.math = Math::Parens;
    }
    match obj.get("rewriteUrls") {
      Some(J::String(r)) => o.rewrite_urls = RewriteUrls::from_name(r).unwrap_or_default(),
      Some(J::Number(n)) => {
        o.rewrite_urls = match n.as_u64() {
          Some(1) => RewriteUrls::Local,
          Some(2) => RewriteUrls::All,
          _ => RewriteUrls::Off,
        }
      }
      _ => {}
    }
    if b("relativeUrls") == Some(true) {
      o.rewrite_urls = RewriteUrls::All;
    }
    o.rootpath = s("rootpath").unwrap_or_default();
    o.url_args = s("urlArgs").unwrap_or_default();
    match obj.get("paths") {
      Some(J::String(p)) => o.include_paths = vec![p.clone()],
      Some(J::Array(a)) => {
        o.include_paths = a
          .iter()
          .filter_map(|x| x.as_str().map(|s| s.to_string()))
          .collect()
      }
      _ => {}
    }
    o.strict_imports = b("strictImports").unwrap_or(false);
    o.javascript_enabled = b("javascriptEnabled").unwrap_or(false);
    o.dump_line_numbers = s("dumpLineNumbers").and_then(|d| DumpLineNumbers::from_name(&d));
    o.global_vars = vars("globalVars");
    o.modify_vars = vars("modifyVars");
    o.banner = s("banner");
    o.process_imports = b("processImports").unwrap_or(true);
    o.quiet = b("quiet").unwrap_or(false);
    o.quiet_deprecations = b("quietDeprecations").unwrap_or(false);
    o.insecure = b("insecure").unwrap_or(false);
    o.disable_plugin_rule = b("disablePluginRule").unwrap_or(false);
    match obj.get("sourceMap") {
      Some(J::Bool(true)) => o.source_map = Some(SourceMapOptions::default()),
      Some(J::Object(m)) => {
        let ss = |k: &str| m.get(k).and_then(|x| x.as_str()).map(|x| x.to_string());
        let bb = |k: &str| m.get(k).and_then(|x| x.as_bool()).unwrap_or(false);
        o.source_map = Some(SourceMapOptions {
          source_map_url: ss("sourceMapURL"),
          source_map_basepath: ss("sourceMapBasepath"),
          source_map_rootpath: ss("sourceMapRootpath"),
          source_map_filename: ss("sourceMapFilename"),
          source_map_output_filename: ss("sourceMapOutputFilename"),
          source_map_input_filename: ss("sourceMapInputFilename"),
          source_map_full_filename: ss("sourceMapFullFilename"),
          source_map_file_inline: bb("sourceMapFileInline"),
          output_source_files: bb("outputSourceFiles"),
          disable_sourcemap_annotation: bb("disableSourcemapAnnotation"),
        });
      }
      _ => {}
    }
    o
  }
}
