use std::collections::HashSet;

pub type AliasMap = nodejs_resolver::AliasMap;

#[derive(Debug, Clone)]
pub struct Resolve {
  /// Tried detect file with this extension.
  /// Default is `[".tsx", ".jsx", ".ts", ".js", ".json"]`
  pub extensions: Vec<String>,
  /// Maps key to value.
  /// Default is `vec![]`.
  /// The reason for using `Vec` instead `HashMap` to keep the order.
  pub alias: Vec<(String, AliasMap)>,
  /// Prefer to resolve request as relative request and
  /// fallback to resolving as modules.
  /// Default is `false`
  pub prefer_relative: bool,
  /// Whether to resolve the real path when the result
  /// is a symlink.
  /// Default is `true`.
  pub symlinks: bool,
  /// Main file in this directory.
  /// Default is `["index"]`.
  pub main_files: Vec<String>,
  /// Main fields in Description.
  /// Default is `["main"]`.
  pub main_fields: Vec<String>,
  /// Whether read and parse `"browser"` filed
  /// in package.json.
  /// Default is `true`
  pub browser_field: bool,
  /// Condition names for exports filed. Note that its
  /// type is a `HashSet`, because the priority is
  /// related to the order in which the export field
  /// fields are written.
  /// Default is `Set()`.
  pub condition_names: HashSet<String>,
}

impl Default for Resolve {
  fn default() -> Self {
    Self {
      extensions: vec![".tsx", ".jsx", ".ts", ".js", ".json"]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
      alias: vec![],
      prefer_relative: false,
      symlinks: true,
      main_files: vec![String::from("index")],
      main_fields: vec![String::from("main")],
      browser_field: true,
      condition_names: HashSet::new(),
    }
  }
}
