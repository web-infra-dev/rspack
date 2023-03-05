use std::{path::PathBuf, sync::Arc};

pub type AliasMap = nodejs_resolver::AliasMap;

pub type Alias = Vec<(String, Vec<AliasMap>)>;

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct Resolve {
  /// Tried detect file with this extension.
  pub extensions: Option<Vec<String>>,
  /// Maps key to value.
  /// The reason for using `Vec` instead `HashMap` to keep the order.
  pub alias: Option<Alias>,
  /// Prefer to resolve request as relative request and
  /// fallback to resolving as modules.
  pub prefer_relative: Option<bool>,
  /// Whether to resolve the real path when the result
  /// is a symlink.
  pub symlinks: Option<bool>,
  /// Main file in this directory.
  pub main_files: Option<Vec<String>>,
  /// Main fields in Description.
  pub main_fields: Option<Vec<String>>,
  /// Whether read and parse `"browser"` filed
  /// in package.json.
  pub browser_field: Option<bool>,
  /// Condition names for exports filed. Note that its
  /// type is a `HashSet`, because the priority is
  /// related to the order in which the export field
  /// fields are written.
  pub condition_names: Option<Vec<String>>,
  /// the path of tsconfig.
  pub tsconfig: Option<PathBuf>,
  /// A list of directories to resolve modules from, can be absolute path or folder name.
  /// Default is `["node_modules"]`
  pub modules: Option<Vec<String>>,
  // Same as `alias`, but only used if default resolving fails
  // Default is `[]`
  pub fallback: Option<Alias>,
}

impl Resolve {
  pub fn to_inner_options(
    self,
    cache: Arc<nodejs_resolver::Cache>,
    resolve_to_context: bool,
  ) -> nodejs_resolver::Options {
    let tsconfig = self.tsconfig;
    let enforce_extension = nodejs_resolver::EnforceExtension::Auto;
    let external_cache = Some(cache);
    let description_file = String::from("package.json");
    let extensions = self.extensions.unwrap_or_else(|| {
      vec![".tsx", ".jsx", ".ts", ".js", ".mjs", ".json"]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    });
    let alias = self.alias.unwrap_or_default();
    let prefer_relative = self.prefer_relative.unwrap_or(false);
    let symlinks = self.symlinks.unwrap_or(true);
    let main_files = self
      .main_files
      .unwrap_or_else(|| vec![String::from("index")]);
    let main_fields = self
      .main_fields
      .unwrap_or_else(|| vec![String::from("module"), String::from("main")]);
    let browser_field = self.browser_field.unwrap_or(true);
    let condition_names = std::collections::HashSet::from_iter(
      self
        .condition_names
        .unwrap_or_else(|| vec!["module".to_string(), "import".to_string()]),
    );
    let modules = self
      .modules
      .unwrap_or_else(|| vec!["node_modules".to_string()]);
    let fallback = self.fallback.unwrap_or_default();
    nodejs_resolver::Options {
      fallback,
      modules,
      extensions,
      enforce_extension,
      alias,
      prefer_relative,
      external_cache,
      symlinks,
      description_file,
      main_files,
      main_fields,
      browser_field,
      condition_names,
      tsconfig,
      resolve_to_context,
    }
  }
}
