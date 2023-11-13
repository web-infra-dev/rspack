use std::{borrow::Cow, path::PathBuf};

use hashlink::LinkedHashMap;

use crate::DependencyCategory;

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
  pub tsconfig: Option<TsconfigOptions>,
  /// A list of directories to resolve modules from, can be absolute path or folder name.
  /// Default is `["node_modules"]`
  pub modules: Option<Vec<String>>,
  /// Same as `alias`, but only used if default resolving fails
  /// Default is `[]`
  pub fallback: Option<Alias>,
  /// Request passed to resolve is already fully specified and
  /// extensions or main files are not resolved for it.
  /// Default is `false`.
  pub fully_specified: Option<bool>,
  /// A list of exports fields in descriptions files
  /// Default is `[["exports"]]`.
  pub exports_field: Option<Vec<Vec<String>>>,
  /// A list map ext to another.
  /// Default is `[]`
  pub extension_alias: Option<Vec<(String, Vec<String>)>>,
  pub by_dependency: Option<ByDependency>,
}

/// Tsconfig Options
///
/// Derived from [tsconfig-paths-webpack-plugin](https://github.com/dividab/tsconfig-paths-webpack-plugin#options)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TsconfigReferences {
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
    let by_dependency = self
      .by_dependency
      .as_ref()
      .and_then(|i| i.get(&dependency_type).cloned());
    self.by_dependency = None;
    if let Some(by_dependency) = by_dependency {
      self = self.merge(by_dependency);
    }
    self
  }

  pub fn merge(self, value: Self) -> Self {
    merge_resolver_options(self, value)
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
  // TODO: maybe a Merge trait for implementing cleverMerge in rust side?
  pub fn merge(mut self, value: Self) -> Self {
    for (k, v) in value.0 {
      let v = if let Some(origin) = self.0.remove(&k) {
        origin.merge(v)
      } else {
        v
      };
      self.0.insert(k, v);
    }
    self
  }

  pub fn get(&self, k: &DependencyCategory) -> Option<&Resolve> {
    self.0.get(k.as_str()).or_else(|| self.0.get("default"))
  }
}

fn merge_resolver_options(base: Resolve, other: Resolve) -> Resolve {
  fn overwrite<T, F>(a: Option<T>, b: Option<T>, f: F) -> Option<T>
  where
    T: Clone,
    F: FnOnce(T, T) -> T,
  {
    match (a, b) {
      (Some(a), Some(b)) => Some(f(a, b)),
      (Some(a), None) => Some(a),
      (None, Some(b)) => Some(b),
      (None, None) => None,
    }
  }

  let alias = overwrite(base.alias, other.alias, |pre, mut now| {
    now.extend(pre);
    now.dedup();
    now
  });
  let fallback = overwrite(base.fallback, other.fallback, |pre, mut now| {
    now.extend(pre);
    now.dedup();
    now
  });
  let prefer_relative = overwrite(base.prefer_relative, other.prefer_relative, |_, value| {
    value
  });
  let symlinks = overwrite(base.symlinks, other.symlinks, |_, value| value);
  let fully_specified = overwrite(base.fully_specified, other.fully_specified, |_, value| {
    value
  });
  let browser_field = overwrite(base.browser_field, other.browser_field, |_, value| value);
  let extensions = overwrite(base.extensions, other.extensions, |base, value| {
    normalize_string_array(&base, value)
  });
  let main_files = overwrite(base.main_files, other.main_files, |base, value| {
    normalize_string_array(&base, value)
  });
  let main_fields = overwrite(base.main_fields, other.main_fields, |base, value| {
    normalize_string_array(&base, value)
  });
  let modules = overwrite(base.modules, other.modules, |base, value| {
    normalize_string_array(&base, value)
  });
  let condition_names = overwrite(
    base.condition_names,
    other.condition_names,
    |base, value| normalize_string_array(&base, value),
  );
  let by_dependency = overwrite(base.by_dependency, other.by_dependency, |pre, now| {
    pre.merge(now)
  });
  let tsconfig = overwrite(base.tsconfig, other.tsconfig, |_, value| value);
  let exports_field = overwrite(base.exports_field, other.exports_field, |_, value| value);
  let extension_alias = overwrite(
    base.extension_alias,
    other.extension_alias,
    |pre, mut now| {
      now.extend(pre);
      now.dedup();
      now
    },
  );
  Resolve {
    fallback,
    modules,
    alias,
    prefer_relative,
    symlinks,
    browser_field,
    extensions,
    main_files,
    main_fields,
    condition_names,
    tsconfig,
    by_dependency,
    fully_specified,
    exports_field,
    extension_alias,
  }
}

fn normalize_string_array(a: &[String], b: Vec<String>) -> Vec<String> {
  b.into_iter().fold(vec![], |mut acc, item| {
    if item.eq("...") {
      acc.append(&mut a.to_vec());
    } else {
      acc.push(item);
    }
    acc
  })
}

#[cfg(test)]
mod test {
  use super::*;

  fn to_string(a: Vec<&str>) -> Vec<String> {
    a.into_iter().map(String::from).collect()
  }

  #[test]
  fn test_merge_resolver_options() {
    use crate::AliasMap;
    let base = Resolve {
      extensions: Some(to_string(vec!["a", "b"])),
      alias: Some(vec![("c".to_string(), vec![AliasMap::Ignored])]),
      symlinks: Some(false),
      main_files: Some(to_string(vec!["d", "e", "f"])),
      main_fields: Some(to_string(vec!["g", "h", "i"])),
      browser_field: Some(true),
      condition_names: Some(to_string(vec!["j", "k"])),
      ..Default::default()
    };
    let another = Resolve {
      extensions: Some(to_string(vec!["a1", "b1"])),
      alias: Some(vec![("c2".to_string(), vec![AliasMap::Ignored])]),
      prefer_relative: Some(true),
      browser_field: Some(true),
      main_files: Some(to_string(vec!["d1", "e", "..."])),
      main_fields: Some(to_string(vec!["...", "h", "..."])),
      condition_names: Some(to_string(vec!["f", "..."])),
      ..Default::default()
    };
    let options = merge_resolver_options(base, another);
    assert_eq!(
      options.extensions.expect("should be Ok"),
      to_string(vec!["a1", "b1"])
    );
    assert!(options.prefer_relative.expect("should be Ok"));
    assert!(!options.symlinks.expect("should be Ok"));
    assert_eq!(
      options.main_files.expect("should be Ok"),
      vec!["d1", "e", "d", "e", "f"]
    );
    assert_eq!(
      options.main_fields.expect("should be Ok"),
      vec!["g", "h", "i", "h", "g", "h", "i"]
    );
    assert_eq!(
      options.alias.expect("should be Ok"),
      vec![
        ("c2".to_string(), vec![AliasMap::Ignored]),
        ("c".to_string(), vec![AliasMap::Ignored])
      ]
    );
    assert_eq!(options.condition_names.expect("should be Ok").len(), 3);
  }

  #[test]
  fn test_normalize_string_array() {
    let base = to_string(vec!["base0", "base1"]);
    assert!(normalize_string_array(&base, vec![]).is_empty());
    assert_eq!(
      normalize_string_array(&base, to_string(vec!["a", "b"])),
      to_string(vec!["a", "b"])
    );
    assert_eq!(
      normalize_string_array(&base, to_string(vec!["...", "a", "...", "b", "..."])),
      to_string(vec![
        "base0", "base1", "a", "base0", "base1", "b", "base0", "base1"
      ])
    );
  }
}
