mod clever_merge;
mod value_type;

use std::{borrow::Cow, path::PathBuf};

use hashlink::LinkedHashMap;

use crate::DependencyCategory;

pub type AliasMap = nodejs_resolver::AliasMap;

pub type Alias = Vec<(String, Vec<AliasMap>)>;
pub(super) type Extensions = Vec<String>;
pub(super) type PreferRelative = bool;
pub(super) type Symlink = bool;
pub(super) type MainFiles = Vec<String>;
pub(super) type MainFields = Vec<String>;
pub(super) type BrowserField = bool;
pub(super) type ConditionNames = Vec<String>;
pub(super) type Fallback = Alias;
pub(super) type FullySpecified = bool;
pub(super) type ExportsField = Vec<Vec<String>>;
pub(super) type ExtensionAlias = Vec<(String, Vec<String>)>;
pub(super) type Modules = Vec<String>;

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct Resolve {
  /// Tried detect file with this extension.
  pub extensions: Option<Extensions>,
  /// Maps key to value.
  /// The reason for using `Vec` instead `HashMap` to keep the order.
  pub alias: Option<Alias>,
  /// Prefer to resolve request as relative request and
  /// fallback to resolving as modules.
  pub prefer_relative: Option<PreferRelative>,
  /// Whether to resolve the real path when the result
  /// is a symlink.
  pub symlinks: Option<Symlink>,
  /// Main file in this directory.
  pub main_files: Option<MainFiles>,
  /// Main fields in Description.
  pub main_fields: Option<MainFields>,
  /// Whether read and parse `"browser"` filed
  /// in package.json.
  pub browser_field: Option<BrowserField>,
  /// Condition names for exports filed. Note that its
  /// type is a `HashSet`, because the priority is
  /// related to the order in which the export field
  /// fields are written.
  pub condition_names: Option<ConditionNames>,
  /// the path of tsconfig.
  pub tsconfig: Option<TsconfigOptions>,
  /// A list of directories to resolve modules from, can be absolute path or folder name.
  /// Default is `["node_modules"]`
  pub modules: Option<Modules>,
  /// Same as `alias`, but only used if default resolving fails
  /// Default is `[]`
  pub fallback: Option<Fallback>,
  /// Request passed to resolve is already fully specified and
  /// extensions or main files are not resolved for it.
  /// Default is `false`.
  pub fully_specified: Option<FullySpecified>,
  /// A list of exports fields in descriptions files
  /// Default is `[["exports"]]`.
  pub exports_field: Option<ExportsField>,
  /// A list map ext to another.
  /// Default is `[]`
  pub extension_alias: Option<ExtensionAlias>,
  pub by_dependency: Option<ByDependency>,
}

/// Tsconfig Options
///
/// Derived from [tsconfig-paths-webpack-plugin](https://github.com/dividab/tsconfig-paths-webpack-plugin#options)
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct TsconfigOptions {
  /// Allows you to specify where to find the TypeScript configuration file.
  /// You may provide
  /// * a relative path to the configuration file. It will be resolved relative to cwd.
  /// * an absolute path to the configuration file.
  pub config_file: PathBuf,

  /// Support for Typescript Project References.
  pub references: TsconfigReferences,
}

impl From<TsconfigOptions> for oxc_resolver::TsconfigOptions {
  fn from(val: TsconfigOptions) -> Self {
    oxc_resolver::TsconfigOptions {
      config_file: val.config_file,
      references: val.references.into(),
    }
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub enum TsconfigReferences {
  #[default]
  Disabled,
  /// Use the `references` field from tsconfig read from `config_file`.
  Auto,
  /// Manually provided relative or absolute path.
  Paths(Vec<PathBuf>),
}

impl From<TsconfigReferences> for oxc_resolver::TsconfigReferences {
  fn from(val: TsconfigReferences) -> Self {
    match val {
      TsconfigReferences::Disabled => oxc_resolver::TsconfigReferences::Disabled,
      TsconfigReferences::Auto => oxc_resolver::TsconfigReferences::Auto,
      TsconfigReferences::Paths(paths) => oxc_resolver::TsconfigReferences::Paths(paths),
    }
  }
}

impl Resolve {
  pub fn merge_by_dependency(mut self, dependency_type: DependencyCategory) -> Self {
    let Some(mut by_dependency) = self.by_dependency.as_mut().map(std::mem::take) else {
      return self;
    };
    let Some(by_value) = by_dependency
      .take(&dependency_type)
      .or_else(|| by_dependency.take_default())
    else {
      return self;
    };
    Self::merge(self, by_value)
  }

  pub fn merge(self, value: Self) -> Self {
    clever_merge::merge_resolve(self, value)
  }
}

type DependencyCategoryStr = Cow<'static, str>;

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct ByDependency(LinkedHashMap<DependencyCategoryStr, Resolve>);

impl FromIterator<(DependencyCategoryStr, Resolve)> for ByDependency {
  fn from_iter<I: IntoIterator<Item = (DependencyCategoryStr, Resolve)>>(i: I) -> Self {
    Self(LinkedHashMap::from_iter(i))
  }
}

impl ByDependency {
  pub fn get(&self, k: &DependencyCategory) -> Option<&Resolve> {
    self.0.get(k.as_str())
  }

  pub fn take(&mut self, k: &DependencyCategory) -> Option<Resolve> {
    self.0.get_mut(k.as_str()).map(std::mem::take)
  }

  pub fn take_default(&mut self) -> Option<Resolve> {
    self.0.get_mut("default").map(std::mem::take)
  }
}
