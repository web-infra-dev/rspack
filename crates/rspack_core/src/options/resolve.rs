use std::{path::PathBuf, sync::Arc};

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
  pub tsconfig: Option<PathBuf>,
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
  pub by_dependency: Option<ByDependency>,
}

impl Resolve {
  pub fn to_inner_options(
    self,
    cache: Arc<nodejs_resolver::Cache>,
    resolve_to_context: bool,
    dependency_type: DependencyCategory,
  ) -> nodejs_resolver::Options {
    let options = self.merge_by_dependency(dependency_type);
    let tsconfig = options.tsconfig;
    let enforce_extension = nodejs_resolver::EnforceExtension::Auto;
    let external_cache = Some(cache);
    let description_file = String::from("package.json");
    let extensions = options.extensions.unwrap_or_else(|| {
      vec![".tsx", ".ts", ".jsx", ".js", ".mjs", ".json"]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    });
    let alias = options.alias.unwrap_or_default();
    let prefer_relative = options.prefer_relative.unwrap_or(false);
    let symlinks = options.symlinks.unwrap_or(true);
    let main_files = options
      .main_files
      .unwrap_or_else(|| vec![String::from("index")]);
    let main_fields = options
      .main_fields
      .unwrap_or_else(|| vec![String::from("module"), String::from("main")]);
    let browser_field = options.browser_field.unwrap_or(true);
    let condition_names = std::collections::HashSet::from_iter(
      options
        .condition_names
        .unwrap_or_else(|| vec!["module".to_string(), "import".to_string()]),
    );
    let modules = options
      .modules
      .unwrap_or_else(|| vec!["node_modules".to_string()]);
    let fallback = options.fallback.unwrap_or_default();
    let fully_specified = options.fully_specified.unwrap_or_default();
    let exports_field = options
      .exports_field
      .unwrap_or_else(|| vec![vec!["exports".to_string()]]);
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
      fully_specified,
      exports_field,
    }
  }

  fn merge_by_dependency(mut self, dependency_type: DependencyCategory) -> Self {
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

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct ByDependency(LinkedHashMap<DependencyCategory, Resolve>);

impl FromIterator<(DependencyCategory, Resolve)> for ByDependency {
  fn from_iter<I: IntoIterator<Item = (DependencyCategory, Resolve)>>(i: I) -> Self {
    Self(LinkedHashMap::from_iter(i.into_iter()))
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
    self.0.get(k)
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
    now.extend(pre.into_iter());
    let now: indexmap::IndexSet<(String, Vec<AliasMap>)> = now.into_iter().collect();
    now.into_iter().collect()
  });
  let fallback = overwrite(base.fallback, other.fallback, |pre, mut now| {
    now.extend(pre.into_iter());
    let now: indexmap::IndexSet<(String, Vec<AliasMap>)> = now.into_iter().collect();
    now.into_iter().collect()
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
