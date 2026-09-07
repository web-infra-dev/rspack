use std::{
  fmt,
  path::{Path, PathBuf},
  sync::Arc,
};

use camino::Utf8Path;

use crate::path::PathUtil;

/// Module Resolution Options
///
/// Options are directly ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve#resolver-options).
///
/// See [webpack resolve](https://webpack.js.org/configuration/resolve/) for information and examples
#[derive(Debug, Clone)]
pub struct ResolveOptions {
  /// Path to TypeScript configuration file.
  ///
  /// Default `None`
  pub tsconfig: Option<TsconfigOptions>,

  /// Create aliases to import or require certain modules more easily.
  ///
  /// An alias is used to replace a whole path or part of a path.
  /// For example, to alias a commonly used `src/` folders: `vec![("@/src"), vec![AliasValue::Path("/path/to/src")]]`
  ///
  /// A trailing $ can also be added to the given object's keys to signify an exact match.
  ///
  /// See [webpack's `resolve.alias` documentation](https://webpack.js.org/configuration/resolve/#resolvealias) for a list of use cases.
  pub alias: Alias,

  /// A list of alias fields in description files.
  ///
  /// Specify a field, such as `browser`, to be parsed according to [this specification](https://github.com/defunctzombie/package-browser-field-spec).
  /// Can be a path to json object such as `["path", "to", "exports"]`.
  ///
  /// Default `[]`
  pub alias_fields: Vec<Vec<String>>,

  /// Condition names for exports field which defines entry points of a package.
  ///
  /// The key order in the exports field is significant. During condition matching, earlier entries have higher priority and take precedence over later entries.
  ///
  /// Default `[]`
  pub condition_names: Vec<String>,

  /// The JSON files to use for descriptions. (There was once a `bower.json`.)
  ///
  /// Default `["package.json"]`
  pub description_files: Vec<String>,

  /// Whether the resolver should check for the presence of a .pnp.cjs file up the dependency tree.
  ///
  /// Default `true`
  #[cfg(feature = "yarn_pnp")]
  pub enable_pnp: bool,

  /// Set to [EnforceExtension::Enabled] for [ESM Mandatory file extensions](https://nodejs.org/api/esm.html#mandatory-file-extensions).
  ///
  /// If `enforce_extension` is set to [EnforceExtension::Enabled], resolution will not allow extension-less files.
  /// This means `require('./foo.js')` will resolve, while `require('./foo')` will not.
  ///
  /// The default value for `enforce_extension` is [EnforceExtension::Auto], which is changed upon initialization.
  ///
  /// It changes to [EnforceExtension::Enabled] if [ResolveOptions::extensions] contains an empty string;
  /// otherwise, this value changes to [EnforceExtension::Disabled].
  ///
  /// Explicitly set the value to [EnforceExtension::Disabled] to disable this automatic behavior.
  ///
  /// For reference, this behavior is aligned with `enhanced-resolve`. See <https://github.com/webpack/enhanced-resolve/pull/285>.
  pub enforce_extension: EnforceExtension,

  /// A list of exports fields in description files.
  ///
  /// Can be a path to a JSON object such as `["path", "to", "exports"]`.
  ///
  /// Default `[["exports"]]`.
  pub exports_fields: Vec<Vec<String>>,

  /// Fields from `package.json` which are used to provide the internal requests of a package
  /// (requests starting with # are considered internal).
  ///
  /// Can be a path to a JSON object such as `["path", "to", "imports"]`.
  ///
  /// Default `[["imports"]]`.
  pub imports_fields: Vec<Vec<String>>,

  /// An object which maps extension to extension aliases.
  ///
  /// Default `{}`
  pub extension_alias: Vec<(String, Vec<String>)>,

  /// Attempt to resolve these extensions in order.
  ///
  /// If multiple files share the same name but have different extensions,
  /// will resolve the one with the extension listed first in the array and skip the rest.
  ///
  /// All extensions must have a leading dot.
  ///
  /// Default `[".js", ".json", ".node"]`
  pub extensions: Vec<String>,

  /// Redirect module requests when normal resolving fails.
  ///
  /// Default `[]`
  pub fallback: Alias,

  /// Request passed to resolve is already fully specified and extensions or main files are not resolved for it (they are still resolved for internal requests).
  ///
  /// See also webpack configuration [resolve.fullySpecified](https://webpack.js.org/configuration/module/#resolvefullyspecified)
  ///
  /// Default `false`
  pub fully_specified: bool,

  /// A list of main fields in description files
  ///
  /// Default `["main"]`.
  pub main_fields: Vec<String>,

  /// The filename to be used while resolving directories.
  ///
  /// Default `["index"]`
  pub main_files: Vec<String>,

  /// A list of directories to resolve modules from, can be absolute path or folder name.
  ///
  /// Default `["node_modules"]`
  pub modules: Vec<String>,

  /// Resolve to a context instead of a file.
  ///
  /// Default `false`
  pub resolve_to_context: bool,

  /// Prefer to resolve module requests as relative requests instead of using modules from node_modules directories.
  ///
  /// Default `false`
  pub prefer_relative: bool,

  /// Prefer to resolve server-relative urls as absolute paths before falling back to resolve in ResolveOptions::roots.
  ///
  /// Default `false`
  pub prefer_absolute: bool,

  /// A list of resolve restrictions to restrict the paths that a request can be resolved on.
  ///
  /// Default `[]`
  pub restrictions: Vec<Restriction>,

  /// A list of directories where requests of server-relative URLs (starting with '/') are resolved.
  /// On non-Windows systems these requests are resolved as an absolute path first.
  ///
  /// Default `[]`
  pub roots: Vec<PathBuf>,

  /// Whether to resolve symlinks to their symlinked location.
  /// When enabled, symlinked resources are resolved to their real path, not their symlinked location.
  /// Note that this may cause module resolution to fail when using tools that symlink packages (like npm link).
  ///
  /// Default `true`
  pub symlinks: bool,

  /// Whether to parse [module.builtinModules](https://nodejs.org/api/module.html#modulebuiltinmodules) or not.
  /// For example, "zlib" will throw [crate::ResolveError::Builtin] when set to true.
  ///
  /// Default `false`
  pub builtin_modules: bool,

  /// When enabled, the resolver also searches
  /// [`NODE_PATH`](https://nodejs.org/api/modules.html#loading-from-the-global-folders)
  /// entries after `node_modules`. Callers can decide when to enable it
  /// (e.g. only for CJS dependency types).
  ///
  /// Default `false`
  pub node_path: bool,
}

impl ResolveOptions {
  /// ## Examples
  ///
  /// ```
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_condition_names(&["bar"]);
  /// assert_eq!(options.condition_names, vec!["bar".to_string()])
  /// ```
  #[must_use]
  pub fn with_condition_names(mut self, names: &[&str]) -> Self {
    self.condition_names = names
      .iter()
      .map(ToString::to_string)
      .collect::<Vec<String>>();
    self
  }

  /// ## Examples
  ///
  /// ```
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_builtin_modules(false);
  /// assert_eq!(options.builtin_modules, false)
  /// ```
  #[must_use]
  pub fn with_builtin_modules(mut self, flag: bool) -> Self {
    self.builtin_modules = flag;
    self
  }

  /// Adds a single root to the options
  ///
  /// ## Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  ///
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_root("foo");
  /// assert_eq!(options.roots, vec![PathBuf::from("foo")])
  /// ```
  #[must_use]
  pub fn with_root<P: AsRef<Path>>(mut self, root: P) -> Self {
    self.roots.push(root.as_ref().to_path_buf());
    self
  }

  /// Adds a single extension to the list of extensions
  ///
  /// ## Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  ///
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_extension("jsonc");
  /// assert!(options.extensions.contains(&"jsonc".to_string()));
  /// ```
  #[must_use]
  pub fn with_extension<S: Into<String>>(mut self, extension: S) -> Self {
    self.extensions.push(extension.into());
    self
  }

  /// Adds a single main field to the list of fields
  ///
  /// ## Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  ///
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_main_field("something");
  /// assert!(options.main_fields.contains(&"something".to_string()));
  /// ```
  #[must_use]
  pub fn with_main_field<S: Into<String>>(mut self, field: S) -> Self {
    self.main_fields.push(field.into());
    self
  }

  /// Changes how the extension should be treated
  ///
  /// ## Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  ///
  /// use rspack_resolver::{EnforceExtension, ResolveOptions};
  ///
  /// let options = ResolveOptions::default().with_force_extension(EnforceExtension::Enabled);
  /// assert_eq!(options.enforce_extension, EnforceExtension::Enabled);
  /// ```
  #[must_use]
  pub fn with_force_extension(mut self, enforce_extension: EnforceExtension) -> Self {
    self.enforce_extension = enforce_extension;
    self
  }

  /// Sets the value for [ResolveOptions::fully_specified]
  ///
  /// ## Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  ///
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_fully_specified(true);
  /// assert_eq!(options.fully_specified, true);
  /// ```
  #[must_use]
  pub fn with_fully_specified(mut self, fully_specified: bool) -> Self {
    self.fully_specified = fully_specified;
    self
  }
  /// Sets the value for [ResolveOptions::prefer_relative]
  ///
  /// ## Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  ///
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_prefer_relative(true);
  /// assert_eq!(options.prefer_relative, true);
  /// ```
  #[must_use]
  pub fn with_prefer_relative(mut self, flag: bool) -> Self {
    self.prefer_relative = flag;
    self
  }
  /// Sets the value for [ResolveOptions::prefer_absolute]
  ///
  /// ## Examples
  ///
  /// ```
  /// use std::path::{Path, PathBuf};
  ///
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_prefer_absolute(true);
  /// assert_eq!(options.prefer_absolute, true);
  /// ```
  #[must_use]
  pub fn with_prefer_absolute(mut self, flag: bool) -> Self {
    self.prefer_absolute = flag;
    self
  }

  /// Changes the value of [ResolveOptions::symlinks]
  ///
  /// ## Examples
  ///
  /// ```
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_symbolic_link(false);
  /// assert_eq!(options.symlinks, false);
  /// ```
  #[must_use]
  pub fn with_symbolic_link(mut self, flag: bool) -> Self {
    self.symlinks = flag;
    self
  }

  /// Adds a module to [ResolveOptions::modules]
  ///
  /// ## Examples
  ///
  /// ```
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_module("module");
  /// assert!(options.modules.contains(&"module".to_string()));
  /// ```
  #[must_use]
  pub fn with_module<M: Into<String>>(mut self, module: M) -> Self {
    self.modules.push(module.into());
    self
  }

  /// Adds a main file to [ResolveOptions::main_files]
  ///
  /// ## Examples
  ///
  /// ```
  /// use rspack_resolver::ResolveOptions;
  ///
  /// let options = ResolveOptions::default().with_main_file("foo");
  /// assert!(options.main_files.contains(&"foo".to_string()));
  /// ```
  #[must_use]
  pub fn with_main_file<M: Into<String>>(mut self, module: M) -> Self {
    self.main_files.push(module.into());
    self
  }

  pub(crate) fn sanitize(mut self) -> Self {
    debug_assert!(
      self
        .extensions
        .iter()
        .filter(|e| !e.is_empty())
        .all(|e| e.starts_with('.')),
      "All extensions must start with a leading dot"
    );
    // Set `enforceExtension` to `true` when [ResolveOptions::extensions] contains an empty string.
    // See <https://github.com/webpack/enhanced-resolve/pull/285>
    if self.enforce_extension == EnforceExtension::Auto {
      if !self.extensions.is_empty() && self.extensions.iter().any(String::is_empty) {
        self.enforce_extension = EnforceExtension::Enabled;
      } else {
        self.enforce_extension = EnforceExtension::Disabled;
      }
    }
    for restriction in &mut self.restrictions {
      if let Restriction::Path(path) = restriction {
        let normalized = Utf8Path::from_path(path).map(PathUtil::normalize);
        if let Some(normalized) = normalized.filter(|p| !p.as_str().is_empty()) {
          *path = normalized.into_std_path_buf();
        }
      }
    }
    self
  }
}

/// Value for [ResolveOptions::enforce_extension]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforceExtension {
  Auto,
  Enabled,
  Disabled,
}

impl EnforceExtension {
  pub const fn is_auto(&self) -> bool {
    matches!(self, Self::Auto)
  }

  pub const fn is_enabled(&self) -> bool {
    matches!(self, Self::Enabled)
  }

  pub const fn is_disabled(&self) -> bool {
    matches!(self, Self::Disabled)
  }
}

/// Alias for [ResolveOptions::alias] and [ResolveOptions::fallback]
pub type Alias = Vec<(String, Vec<AliasValue>)>;

/// Alias Value for [ResolveOptions::alias] and [ResolveOptions::fallback]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AliasValue {
  /// The path value
  Path(String),

  /// The `false` value
  Ignore,
}

impl<S> From<S> for AliasValue
where
  S: Into<String>,
{
  fn from(value: S) -> Self {
    Self::Path(value.into())
  }
}

/// Value for [ResolveOptions::restrictions]
#[derive(Clone)]
pub enum Restriction {
  Path(PathBuf),
  Fn(Arc<dyn Fn(&Path) -> bool + Sync + Send>),
}

impl std::fmt::Debug for Restriction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Path(path) => write!(f, "Path({path:?})"),
      Self::Fn(_) => write!(f, "Fn(<function>)"),
    }
  }
}

/// Tsconfig Options for [ResolveOptions::tsconfig]
///
/// Derived from [tsconfig-paths-webpack-plugin](https://github.com/dividab/tsconfig-paths-webpack-plugin#options)
#[derive(Debug, Clone)]
pub struct TsconfigOptions {
  /// Allows you to specify where to find the TypeScript configuration file.
  /// You may provide
  /// * a relative path to the configuration file. It will be resolved relative to cwd.
  /// * an absolute path to the configuration file.
  pub config_file: PathBuf,

  /// Support for Typescript Project References.
  pub references: TsconfigReferences,
}

/// Configuration for [TsconfigOptions::references]
#[derive(Debug, Clone)]
pub enum TsconfigReferences {
  Disabled,
  /// Use the `references` field from tsconfig of `config_file`.
  Auto,
  /// Manually provided relative or absolute path.
  Paths(Vec<PathBuf>),
}

impl Default for ResolveOptions {
  fn default() -> Self {
    Self {
      tsconfig: None,
      alias: vec![],
      alias_fields: vec![],
      condition_names: vec![],
      description_files: vec!["package.json".into()],
      enforce_extension: EnforceExtension::Auto,
      extension_alias: vec![],
      exports_fields: vec![vec!["exports".into()]],
      imports_fields: vec![vec!["imports".into()]],
      extensions: vec![".js".into(), ".json".into(), ".node".into()],
      fallback: vec![],
      fully_specified: false,
      main_fields: vec!["main".into()],
      main_files: vec!["index".into()],
      modules: vec!["node_modules".into()],
      #[cfg(feature = "yarn_pnp")]
      enable_pnp: true,
      resolve_to_context: false,
      prefer_relative: false,
      prefer_absolute: false,
      restrictions: vec![],
      roots: vec![],
      symlinks: true,
      builtin_modules: false,
      node_path: false,
    }
  }
}

// For tracing
impl fmt::Display for ResolveOptions {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(tsconfig) = &self.tsconfig {
      write!(f, "tsconfig:{tsconfig:?},")?;
    }
    if !self.alias.is_empty() {
      write!(f, "alias:{:?},", self.alias)?;
    }
    if !self.alias_fields.is_empty() {
      write!(f, "alias_fields:{:?},", self.alias_fields)?;
    }
    if !self.condition_names.is_empty() {
      write!(f, "condition_names:{:?},", self.condition_names)?;
    }
    if self.enforce_extension.is_enabled() {
      write!(f, "enforce_extension:{:?},", self.enforce_extension)?;
    }
    if !self.exports_fields.is_empty() {
      write!(f, "exports_fields:{:?},", self.exports_fields)?;
    }
    if !self.imports_fields.is_empty() {
      write!(f, "imports_fields:{:?},", self.imports_fields)?;
    }
    if !self.extension_alias.is_empty() {
      write!(f, "extension_alias:{:?},", self.extension_alias)?;
    }
    if !self.extensions.is_empty() {
      write!(f, "extensions:{:?},", self.extensions)?;
    }
    if !self.fallback.is_empty() {
      write!(f, "fallback:{:?},", self.fallback)?;
    }
    if self.fully_specified {
      write!(f, "fully_specified:{:?},", self.fully_specified)?;
    }
    if !self.main_fields.is_empty() {
      write!(f, "main_fields:{:?},", self.main_fields)?;
    }
    if !self.main_files.is_empty() {
      write!(f, "main_files:{:?},", self.main_files)?;
    }
    if !self.modules.is_empty() {
      write!(f, "modules:{:?},", self.modules)?;
    }
    if self.resolve_to_context {
      write!(f, "resolve_to_context:{:?},", self.resolve_to_context)?;
    }
    if self.prefer_relative {
      write!(f, "prefer_relative:{:?},", self.prefer_relative)?;
    }
    if self.prefer_absolute {
      write!(f, "prefer_absolute:{:?},", self.prefer_absolute)?;
    }
    if !self.restrictions.is_empty() {
      write!(f, "restrictions:{:?},", self.restrictions)?;
    }
    if !self.roots.is_empty() {
      write!(f, "roots:{:?},", self.roots)?;
    }
    if self.symlinks {
      write!(f, "symlinks:{:?},", self.symlinks)?;
    }
    if self.builtin_modules {
      write!(f, "builtin_modules:{:?},", self.builtin_modules)?;
    }
    if self.node_path {
      write!(f, "node_path:{:?},", self.node_path)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use super::{
    AliasValue, EnforceExtension, ResolveOptions, Restriction, TsconfigOptions, TsconfigReferences,
  };

  #[test]
  fn enforce_extension() {
    assert!(EnforceExtension::Auto.is_auto());
    assert!(!EnforceExtension::Enabled.is_auto());
    assert!(!EnforceExtension::Disabled.is_auto());

    assert!(!EnforceExtension::Auto.is_enabled());
    assert!(EnforceExtension::Enabled.is_enabled());
    assert!(!EnforceExtension::Disabled.is_enabled());

    assert!(!EnforceExtension::Auto.is_disabled());
    assert!(!EnforceExtension::Enabled.is_disabled());
    assert!(EnforceExtension::Disabled.is_disabled());
  }

  #[test]
  fn display() {
    let options = ResolveOptions {
      tsconfig: Some(TsconfigOptions {
        config_file: PathBuf::from("tsconfig.json"),
        references: TsconfigReferences::Auto,
      }),
      alias: vec![("a".into(), vec![AliasValue::Ignore])],
      alias_fields: vec![vec!["browser".into()]],
      condition_names: vec!["require".into()],
      enforce_extension: EnforceExtension::Enabled,
      extension_alias: vec![(".js".into(), vec![".ts".into()])],
      exports_fields: vec![vec!["exports".into()]],
      imports_fields: vec![vec!["imports".into()]],
      fallback: vec![("fallback".into(), vec![AliasValue::Ignore])],
      fully_specified: true,
      resolve_to_context: true,
      prefer_relative: true,
      prefer_absolute: true,
      restrictions: vec![Restriction::Path(PathBuf::from("restrictions"))],
      roots: vec![PathBuf::from("roots")],
      builtin_modules: true,
      ..ResolveOptions::default()
    };

    let expected = r#"tsconfig:TsconfigOptions { config_file: "tsconfig.json", references: Auto },alias:[("a", [Ignore])],alias_fields:[["browser"]],condition_names:["require"],enforce_extension:Enabled,exports_fields:[["exports"]],imports_fields:[["imports"]],extension_alias:[(".js", [".ts"])],extensions:[".js", ".json", ".node"],fallback:[("fallback", [Ignore])],fully_specified:true,main_fields:["main"],main_files:["index"],modules:["node_modules"],resolve_to_context:true,prefer_relative:true,prefer_absolute:true,restrictions:[Path("restrictions")],roots:["roots"],symlinks:true,builtin_modules:true,"#;
    assert_eq!(format!("{options}"), expected);

    let options = ResolveOptions {
      alias: vec![],
      alias_fields: vec![],
      builtin_modules: false,
      condition_names: vec![],
      description_files: vec![],
      #[cfg(feature = "yarn_pnp")]
      enable_pnp: true,
      enforce_extension: EnforceExtension::Disabled,
      exports_fields: vec![],
      extension_alias: vec![],
      extensions: vec![],
      fallback: vec![],
      fully_specified: false,
      imports_fields: vec![],
      main_fields: vec![],
      main_files: vec![],
      modules: vec![],
      prefer_absolute: false,
      prefer_relative: false,
      resolve_to_context: false,
      restrictions: vec![],
      roots: vec![],
      symlinks: false,
      tsconfig: None,
      node_path: false,
    };

    assert_eq!(format!("{options}"), "");
  }
}
