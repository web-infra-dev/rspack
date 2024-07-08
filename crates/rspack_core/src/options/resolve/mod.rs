mod clever_merge;
mod value_type;

use std::{borrow::Cow, path::PathBuf};

use hashlink::LinkedHashMap;

use crate::DependencyCategory;

pub type AliasMap = rspack_resolver::AliasValue;
pub type Alias = rspack_resolver::Alias;

pub(super) type Extensions = Vec<String>;
pub(super) type PreferRelative = bool;
pub(super) type PreferAbsolute = bool;
pub(super) type Symlink = bool;
pub(super) type MainFiles = Vec<String>;
pub(super) type MainFields = Vec<String>;
pub(super) type DescriptionFiles = Vec<String>;
pub(super) type AliasFields = Vec<Vec<String>>;
pub(super) type ConditionNames = Vec<String>;
pub(super) type Fallback = Alias;
pub(super) type FullySpecified = bool;
pub(super) type EnforceExtension = bool;
pub(super) type ExportsFields = Vec<Vec<String>>;
pub(super) type ImportsFields = Vec<Vec<String>>;
pub(super) type ExtensionAlias = Vec<(String, Vec<String>)>;
pub(super) type Modules = Vec<String>;
pub(super) type Roots = Vec<String>;
pub(super) type Restrictions = Vec<String>;

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
  /// Prefer absolute paths to `resolve.roots` when resolving.
  pub prefer_absolute: Option<PreferAbsolute>,
  /// Whether to resolve the real path when the result
  /// is a symlink.
  pub symlinks: Option<Symlink>,
  /// Main file in this directory.
  pub main_files: Option<MainFiles>,
  /// Main fields in Description.
  pub main_fields: Option<MainFields>,
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
  pub exports_fields: Option<ExportsFields>,
  /// A list map ext to another.
  /// Default is `[]`
  pub extension_alias: Option<ExtensionAlias>,
  /// Specify a field, such as browser, to be parsed according to [this specification](https://github.com/defunctzombie/package-browser-field-spec).
  pub alias_fields: Option<AliasFields>,
  /// A list of directories where requests of server-relative URLs (starting with '/') are resolved
  pub roots: Option<Roots>,
  /// A list of resolve restrictions to restrict the paths that a request can be resolved on.
  pub restrictions: Option<Restrictions>,
  /// Field names from the description file (usually package.json) which are used to provide internal request of a package (requests starting with # are considered as internal).
  pub imports_fields: Option<ImportsFields>,
  /// Configure resolve options by the type of module request.
  pub by_dependency: Option<ByDependency>,
  /// The JSON files to use for descriptions
  /// Default is ["package.json"]
  pub description_files: Option<DescriptionFiles>,
  /// If enforce_extension is set to EnforceExtension::Enabled, resolution will not allow extension-less files. This means require('./foo.js') will resolve, while require('./foo') will not.
  pub enforce_extension: Option<EnforceExtension>,
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

impl From<TsconfigOptions> for rspack_resolver::TsconfigOptions {
  fn from(val: TsconfigOptions) -> Self {
    rspack_resolver::TsconfigOptions {
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

impl From<TsconfigReferences> for rspack_resolver::TsconfigReferences {
  fn from(val: TsconfigReferences) -> Self {
    match val {
      TsconfigReferences::Disabled => rspack_resolver::TsconfigReferences::Disabled,
      TsconfigReferences::Auto => rspack_resolver::TsconfigReferences::Auto,
      TsconfigReferences::Paths(paths) => rspack_resolver::TsconfigReferences::Paths(paths),
    }
  }
}

macro_rules! impl_resolve_by_dependency {
  ($ident:ident) => {
    pub fn $ident(&self, cat: Option<&DependencyCategory>) -> Option<bool> {
      cat
        .and_then(|cat| {
          self
            .by_dependency
            .as_ref()
            .and_then(|by_dep| by_dep.get(cat).and_then(|d| d.$ident))
        })
        .or(self.$ident)
    }
  };
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

  impl_resolve_by_dependency!(fully_specified);
  impl_resolve_by_dependency!(prefer_relative);
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
