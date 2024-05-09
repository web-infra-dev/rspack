use hashlink::LinkedHashMap;

use super::value_type::{GetValueType, ValueType};
use super::{
  Alias, AliasFields, ConditionNames, DescriptionFiles, EnforceExtension, ExportsField,
  ExtensionAlias, Extensions, Fallback, FullySpecified, ImportsField, MainFields, MainFiles,
  Modules, PreferAbsolute, PreferRelative, Restrictions, Roots, Symlink, TsconfigOptions,
};
use super::{ByDependency, DependencyCategoryStr, Resolve};

pub(super) fn merge_resolve(first: Resolve, second: Resolve) -> Resolve {
  _merge_resolve(first, second)
}

fn is_empty(resolve: &Resolve) -> bool {
  macro_rules! is_none {
    ($ident: ident) => {
      resolve.$ident.is_none()
    };
  }

  is_none!(extensions)
    && is_none!(alias)
    && is_none!(prefer_relative)
    && is_none!(prefer_absolute)
    && is_none!(symlinks)
    && is_none!(main_files)
    && is_none!(main_fields)
    && is_none!(condition_names)
    && is_none!(modules)
    && is_none!(fallback)
    && is_none!(fully_specified)
    && is_none!(exports_field)
    && is_none!(imports_field)
    && is_none!(description_files)
    && is_none!(enforce_extension)
    && is_none!(extension_alias)
    && is_none!(alias_fields)
    && is_none!(restrictions)
    && is_none!(roots)
    && is_none!(tsconfig)
    && is_none!(by_dependency)
}

#[derive(Default, Debug)]
struct Entry<T: Default + std::fmt::Debug> {
  base: Option<T>,
  by_values: Option<LinkedHashMap<DependencyCategoryStr, Option<T>>>,
}

#[derive(Debug)]
struct ResolveWithEntry {
  extensions: Entry<Extensions>,
  alias: Entry<Alias>,
  prefer_relative: Entry<PreferRelative>,
  prefer_absolute: Entry<PreferAbsolute>,
  symlinks: Entry<Symlink>,
  main_files: Entry<MainFiles>,
  main_fields: Entry<MainFields>,
  condition_names: Entry<ConditionNames>,
  modules: Entry<Modules>,
  fallback: Entry<Fallback>,
  tsconfig: Entry<TsconfigOptions>,
  fully_specified: Entry<FullySpecified>,
  exports_field: Entry<ExportsField>,
  imports_field: Entry<ImportsField>,
  description_files: Entry<DescriptionFiles>,
  enforce_extension: Entry<EnforceExtension>,
  extension_alias: Entry<ExtensionAlias>,
  alias_fields: Entry<AliasFields>,
  restrictions: Entry<Restrictions>,
  roots: Entry<Roots>,
}

fn parse_resolve(resolve: Resolve) -> ResolveWithEntry {
  macro_rules! entry {
    ($ident: ident) => {
      Entry {
        base: resolve.$ident,
        by_values: None,
      }
    };
  }
  let mut res = ResolveWithEntry {
    extensions: entry!(extensions),
    alias: entry!(alias),
    prefer_relative: entry!(prefer_relative),
    prefer_absolute: entry!(prefer_absolute),
    symlinks: entry!(symlinks),
    main_files: entry!(main_files),
    main_fields: entry!(main_fields),
    condition_names: entry!(condition_names),
    modules: entry!(modules),
    fallback: entry!(fallback),
    tsconfig: entry!(tsconfig),
    fully_specified: entry!(fully_specified),
    exports_field: entry!(exports_field),
    imports_field: entry!(imports_field),
    description_files: entry!(description_files),
    enforce_extension: entry!(enforce_extension),
    extension_alias: entry!(extension_alias),
    alias_fields: entry!(alias_fields),
    restrictions: entry!(restrictions),
    roots: entry!(roots),
  };
  let Some(by_dependency) = resolve.by_dependency else {
    return res;
  };
  let mut by_dependency = by_dependency;

  macro_rules! update_by_value {
    ($ident: ident) => {
      let mut $ident = LinkedHashMap::new();
      let by_values_key: Vec<_> = by_dependency.0.keys().cloned().collect();
      for by_value_key in &by_values_key {
        let obj = by_dependency.0.get_mut(by_value_key).expect("");
        if obj.$ident.is_some() {
          $ident.insert(by_value_key.clone(), std::mem::take(&mut obj.$ident));
        }
        if by_value_key == "default" {
          for other_by_value_key in &by_values_key {
            if !$ident.contains_key(other_by_value_key) {
              $ident.insert(other_by_value_key.clone(), None);
            }
          }
        }
      }
      if $ident.len() > 0 {
        res.$ident.by_values = Some($ident);
      }
    };
  }
  update_by_value!(extensions);
  update_by_value!(alias);
  update_by_value!(prefer_relative);
  update_by_value!(prefer_absolute);
  update_by_value!(symlinks);
  update_by_value!(main_files);
  update_by_value!(main_fields);
  update_by_value!(condition_names);
  update_by_value!(modules);
  update_by_value!(fallback);
  update_by_value!(fully_specified);
  update_by_value!(exports_field);
  update_by_value!(imports_field);
  update_by_value!(description_files);
  update_by_value!(enforce_extension);
  update_by_value!(extension_alias);
  update_by_value!(alias_fields);
  update_by_value!(restrictions);
  update_by_value!(roots);
  update_by_value!(tsconfig);

  res
}

fn overwrite<T, F>(a: Option<T>, b: Option<T>, f: F) -> Option<T>
where
  F: FnOnce(&T, T) -> T,
{
  match (a, b) {
    (Some(a), Some(b)) => Some(f(&a, b)),
    (Some(a), None) => Some(a),
    (None, Some(b)) => Some(b),
    (None, None) => None,
  }
}

fn get_from_by_values<T: Default + Clone>(
  by_values: &LinkedHashMap<DependencyCategoryStr, T>,
  key: &str,
) -> Option<T> {
  let value = if key != "default" && by_values.contains_key(key) {
    by_values.get(key)
  } else {
    by_values.get("default")
  };
  // FIXME: not use clone
  value.cloned()
}

fn _merge_resolve(first: Resolve, second: Resolve) -> Resolve {
  let first = parse_resolve(first);
  let second = parse_resolve(second);

  macro_rules! merge {
    ($ident: ident, $second_value_type: expr, $need_merge_base: expr, $deal_merge: expr) => {{
      if second.$ident.base.is_none() {
        if let Some(by_values) = first.$ident.by_values {
          let mut new_by_values = by_values;
          for (key, value) in second.$ident.by_values.unwrap_or_default() {
            let first_value = get_from_by_values(&new_by_values, key.as_ref()).unwrap_or_default();
            new_by_values.insert(key, overwrite(first_value, value, $deal_merge));
          }
          Entry {
            base: first.$ident.base,
            by_values: Some(new_by_values),
          }
        } else {
          // this arm same as `!firstEntry.byProperty``
          Entry {
            base: first.$ident.base,
            by_values: second.$ident.by_values,
          }
        }
      } else if matches!($second_value_type, ValueType::Atom) {
        Entry {
          base: second.$ident.base,
          by_values: None,
        }
      } else if let Some(intermediate_by_values) = first.$ident.by_values {
        #[allow(clippy::redundant_closure_call)]
        let need_merge_base = $need_merge_base(&intermediate_by_values);
        let mut intermediate_by_values: LinkedHashMap<_, _> = intermediate_by_values
          .into_iter()
          .map(|(key, value)| {
            let value = overwrite(value, second.$ident.base.clone(), $deal_merge);
            (key, value)
          })
          .collect();
        let new_base = if need_merge_base {
          overwrite(first.$ident.base, second.$ident.base, $deal_merge)
        } else {
          if !intermediate_by_values.contains_key("default") {
            intermediate_by_values.insert("default".into(), second.$ident.base);
          }
          first.$ident.base
        };

        let new_by_values = if let Some(by_values) = second.$ident.by_values {
          let mut new_by_values = intermediate_by_values;
          for (key, value) in by_values {
            let first_value =
              get_from_by_values(&mut new_by_values, key.as_ref()).unwrap_or_default();
            new_by_values.insert(key, overwrite(first_value, value, $deal_merge));
          }
          new_by_values
        } else {
          intermediate_by_values
        };

        Entry {
          base: new_base,
          by_values: Some(new_by_values),
        }
      } else {
        Entry {
          base: overwrite(first.$ident.base, second.$ident.base, $deal_merge),
          by_values: second.$ident.by_values,
        }
      }
    }};
  }

  let need_merge_base = |by_values: &LinkedHashMap<DependencyCategoryStr, Option<Vec<String>>>| {
    by_values.values().all(|value| {
      let value_type = value.get_value_type();
      assert!(!matches!(value_type, ValueType::Other));
      !matches!(value_type, ValueType::Extend)
    })
  };

  let result_entry = ResolveWithEntry {
    extensions: merge!(
      extensions,
      second.extensions.base.get_value_type(),
      need_merge_base,
      |a, b| normalize_string_array(a, b)
    ),
    prefer_relative: merge!(
      prefer_relative,
      second.prefer_relative.base.get_value_type(),
      |_| true,
      |_, b| b
    ),
    prefer_absolute: merge!(
      prefer_absolute,
      second.prefer_absolute.base.get_value_type(),
      |_| true,
      |_, b| b
    ),
    symlinks: merge!(
      symlinks,
      second.symlinks.base.get_value_type(),
      |_| true,
      |_, b| b
    ),
    main_files: merge!(
      main_files,
      second.main_files.base.get_value_type(),
      need_merge_base,
      |a, b| normalize_string_array(a, b)
    ),
    main_fields: merge!(
      main_fields,
      second.main_fields.base.get_value_type(),
      need_merge_base,
      |a, b| normalize_string_array(a, b)
    ),
    condition_names: merge!(
      condition_names,
      second.condition_names.base.get_value_type(),
      need_merge_base,
      |a, b| normalize_string_array(a, b)
    ),
    modules: merge!(
      modules,
      second.modules.base.get_value_type(),
      need_merge_base,
      |a, b| normalize_string_array(a, b)
    ),
    fully_specified: merge!(
      fully_specified,
      second.fully_specified.base.get_value_type(),
      |_| true,
      |_, b| b
    ),
    fallback: merge!(fallback, ValueType::Other, |_| false, extend_alias),
    alias: merge!(alias, ValueType::Other, |_| false, extend_alias),
    exports_field: merge!(exports_field, ValueType::Other, |_| false, |_, b| b),
    imports_field: merge!(imports_field, ValueType::Other, |_| false, |_, b| b),
    description_files: merge!(
      description_files,
      second.description_files.base.get_value_type(),
      need_merge_base,
      |a, b| normalize_string_array(a, b)
    ),
    enforce_extension: merge!(
      enforce_extension,
      second.enforce_extension.base.get_value_type(),
      |_| true,
      |_, b| b
    ),
    tsconfig: merge!(tsconfig, ValueType::Other, |_| false, |_, b| b),
    extension_alias: merge!(extension_alias, ValueType::Other, |_| false, |a, b| {
      extend_extension_alias(a, b)
    }),
    alias_fields: merge!(alias_fields, ValueType::Other, |_| false, |_, b| b),
    restrictions: merge!(restrictions, ValueType::Other, |_| false, |_, b| b),
    roots: merge!(roots, ValueType::Other, |_| false, |_, b| b),
  };

  let mut by_dependency: LinkedHashMap<DependencyCategoryStr, Resolve> = LinkedHashMap::new();

  macro_rules! setup_by_values {
    ($ident: ident) => {
      if let Some(by_values) = &result_entry.$ident.by_values {
        for key in by_values.keys() {
          if !by_dependency.contains_key(key) {
            by_dependency.insert(key.clone(), Resolve::default());
          }
        }
      }
    };
  }

  setup_by_values!(extensions);
  setup_by_values!(alias);
  setup_by_values!(prefer_relative);
  setup_by_values!(prefer_absolute);
  setup_by_values!(symlinks);
  setup_by_values!(main_files);
  setup_by_values!(main_fields);
  setup_by_values!(condition_names);
  setup_by_values!(tsconfig);
  setup_by_values!(modules);
  setup_by_values!(fallback);
  setup_by_values!(fully_specified);
  setup_by_values!(exports_field);
  setup_by_values!(imports_field);
  setup_by_values!(description_files);
  setup_by_values!(enforce_extension);
  setup_by_values!(extension_alias);
  setup_by_values!(alias_fields);
  setup_by_values!(restrictions);
  setup_by_values!(roots);

  macro_rules! to_resolve {
    ($ident: ident) => {
      if let Some(by_values) = result_entry.$ident.by_values {
        for (key, resolve) in by_dependency.iter_mut() {
          if let Some(value) = get_from_by_values(&by_values, key) {
            resolve.$ident = value;
          }
        }
      }
    };
  }

  to_resolve!(extensions);
  to_resolve!(alias);
  to_resolve!(prefer_relative);
  to_resolve!(prefer_absolute);
  to_resolve!(symlinks);
  to_resolve!(main_files);
  to_resolve!(main_fields);
  to_resolve!(condition_names);
  to_resolve!(tsconfig);
  to_resolve!(modules);
  to_resolve!(fallback);
  to_resolve!(fully_specified);
  to_resolve!(exports_field);
  to_resolve!(imports_field);
  to_resolve!(description_files);
  to_resolve!(enforce_extension);
  to_resolve!(extension_alias);
  to_resolve!(alias_fields);
  to_resolve!(restrictions);
  to_resolve!(roots);

  let by_dependency = if by_dependency.iter().all(|(_, by_value)| is_empty(by_value)) {
    None
  } else {
    Some(ByDependency(by_dependency))
  };

  Resolve {
    by_dependency,
    extensions: result_entry.extensions.base,
    alias: result_entry.alias.base,
    prefer_relative: result_entry.prefer_relative.base,
    prefer_absolute: result_entry.prefer_absolute.base,
    symlinks: result_entry.symlinks.base,
    main_files: result_entry.main_files.base,
    main_fields: result_entry.main_fields.base,
    condition_names: result_entry.condition_names.base,
    tsconfig: result_entry.tsconfig.base,
    modules: result_entry.modules.base,
    fallback: result_entry.fallback.base,
    fully_specified: result_entry.fully_specified.base,
    exports_field: result_entry.exports_field.base,
    imports_field: result_entry.imports_field.base,
    description_files: result_entry.description_files.base,
    enforce_extension: result_entry.enforce_extension.base,
    extension_alias: result_entry.extension_alias.base,
    alias_fields: result_entry.alias_fields.base,
    restrictions: result_entry.restrictions.base,
    roots: result_entry.roots.base,
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

fn extend_alias(a: &Alias, b: Alias) -> Alias {
  let mut b = b;
  // FIXME: I think this clone can be removed
  b.extend(a.clone());
  b.dedup();
  b
}

fn extend_extension_alias(a: &ExtensionAlias, b: ExtensionAlias) -> ExtensionAlias {
  let mut b = b;
  // FIXME: I think this clone can be removed
  b.extend(a.clone());
  b.dedup();
  b
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::AliasMap;

  fn string_list(a: &[&str]) -> Option<Vec<String>> {
    Some(a.iter().map(|s| s.to_string()).collect())
  }

  fn first_case_1() -> Resolve {
    Resolve {
      extensions: string_list(&["1"]),
      ..Default::default()
    }
  }

  fn first_case_2() -> Resolve {
    Resolve {
      extensions: string_list(&["1"]),
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  fn first_case_3() -> Resolve {
    Resolve {
      extensions: string_list(&["1"]),
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5", "...", "6"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  fn first_case_4() -> Resolve {
    Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  fn first_case_5() -> Resolve {
    Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5", "...", "6"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  fn first_case_6() -> Resolve {
    Resolve {
      extensions: string_list(&["1", "...", "2"]),
      ..Default::default()
    }
  }

  fn first_case_7() -> Resolve {
    Resolve {
      extensions: string_list(&["1", "...", "2"]),
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  fn first_case_8() -> Resolve {
    Resolve {
      extensions: string_list(&["1", "...", "2"]),
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5", "...", "6"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  fn first_case_9() -> Resolve {
    Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  fn first_case_10() -> Resolve {
    Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["5", "...", "6"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    }
  }

  #[test]
  fn test_merge_resolver_options_0() {
    let base = Resolve {
      extensions: string_list(&["a", "b"]),
      alias: Some(vec![("c".to_string(), vec![AliasMap::Ignore])]),
      symlinks: Some(false),
      main_files: string_list(&["d", "e", "f"]),
      main_fields: string_list(&["g", "h", "i"]),
      condition_names: string_list(&["j", "k"]),
      ..Default::default()
    };
    let another = Resolve {
      extensions: string_list(&["a1", "b1"]),
      alias: Some(vec![("c2".to_string(), vec![AliasMap::Ignore])]),
      prefer_relative: Some(true),
      main_files: string_list(&["d1", "e", "..."]),
      main_fields: string_list(&["...", "h", "..."]),
      condition_names: string_list(&["f", "..."]),
      ..Default::default()
    };
    let options = merge_resolve(base, another);
    assert_eq!(options.extensions.expect("should be Ok"), vec!["a1", "b1"]);
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
        ("c2".to_string(), vec![AliasMap::Ignore]),
        ("c".to_string(), vec![AliasMap::Ignore])
      ]
    );
    assert_eq!(options.condition_names.expect("should be Ok").len(), 3);
  }

  #[test]
  fn test_merge_resolver_options_1() {
    let first = Resolve {
      extensions: string_list(&["1"]),
      ..Default::default()
    };
    let second = Resolve {
      modules: string_list(&["2"]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first, second),
      Resolve {
        extensions: string_list(&["1"]),
        modules: string_list(&["2"]),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_2() {
    let first = Resolve {
      extensions: string_list(&["1"]),
      ..Default::default()
    };
    let second = Resolve {
      extensions: string_list(&["2"]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first, second),
      Resolve {
        extensions: string_list(&["2"]),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_3() {
    let first = Resolve {
      extensions: string_list(&["1"]),
      modules: string_list(&["1"]),
      alias: Some(vec![]),
      ..Default::default()
    };
    let second = Resolve {
      extensions: string_list(&["2"]),
      modules: string_list(&["2", "...", "3"]),
      alias: Some(vec![("2".to_string(), vec![AliasMap::Ignore])]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first, second),
      Resolve {
        extensions: string_list(&["2"]),
        modules: string_list(&["2", "1", "3"]),
        alias: Some(vec![("2".to_string(), vec![AliasMap::Ignore])]),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_4() {
    let first = Resolve {
      extensions: string_list(&["1"]),
      modules: string_list(&["1"]),
      main_fields: string_list(&["1"]),
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          modules: string_list(&["5"]),
          main_fields: string_list(&["5", "...", "6"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    };
    let second = Resolve {
      extensions: string_list(&["8"]),
      modules: string_list(&["8"]),
      main_fields: string_list(&["8"]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first, second),
      Resolve {
        extensions: string_list(&["8"]),
        modules: string_list(&["8"]),
        main_fields: string_list(&["8"]),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_5() {
    let first = Resolve {
      extensions: string_list(&["1"]),
      modules: string_list(&["1"]),
      main_fields: string_list(&["1"]),
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          modules: string_list(&["5"]),
          main_fields: string_list(&["5", "...", "6"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    };
    let second = Resolve {
      extensions: string_list(&["8", "..."]),
      modules: string_list(&["8", "..."]),
      main_fields: string_list(&["8", "..."]),
      condition_names: string_list(&["8", "..."]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first, second),
      Resolve {
        extensions: string_list(&["8", "1"]),
        modules: string_list(&["8", "1"]),
        main_fields: string_list(&["1"]),
        condition_names: string_list(&["8", "..."]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              modules: string_list(&["8", "5"]),
              main_fields: string_list(&["8", "5", "...", "6"]),
              ..Default::default()
            }
          ),
          (
            "default".into(),
            Resolve {
              main_fields: string_list(&["8", "..."]),
              ..Default::default()
            }
          )
        ])),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_6() {
    let second = Resolve {
      extensions: string_list(&["8", "..."]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first_case_5(), second),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "5", "...", "6"]),
              ..Default::default()
            }
          ),
          (
            "default".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            }
          )
        ])),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_7() {
    let second = Resolve {
      extensions: string_list(&["8", "..."]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first_case_6(), second),
      Resolve {
        extensions: string_list(&["8", "1", "...", "2"]),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_8() {
    let second = Resolve {
      extensions: string_list(&["8", "..."]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first_case_7(), second),
      Resolve {
        extensions: string_list(&["8", "1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "5"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_9() {
    let second = Resolve {
      extensions: string_list(&["8", "..."]),
      ..Default::default()
    };
    pretty_assertions::assert_eq!(
      merge_resolve(first_case_8(), second),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "default".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_10() {
    pretty_assertions::assert_eq!(
      merge_resolve(first_case_9(), Resolve::default()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["5"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_11() {
    pretty_assertions::assert_eq!(
      merge_resolve(first_case_10(), Resolve::default()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["5", "...", "6"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    )
  }

  #[test]
  fn test_merge_resolver_options_12() {
    let second = || Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "y".into(),
        Resolve {
          extensions: string_list(&["8"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    };

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_1(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "y".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_2(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_3(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_4(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_5(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_6(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "y".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_7(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_8(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_9(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_10(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8"]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );
  }

  #[test]
  fn test_merge_resolver_options_13() {
    let second = || Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["8"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    };

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_1(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_2(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        ),])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_3(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_4(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_5(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_6(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_7(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_8(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_9(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_10(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );
  }

  #[test]
  fn test_merge_resolver_options_14() {
    let second = || Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "y".into(),
        Resolve {
          extensions: string_list(&["8", "..."]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    };

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_1(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "y".into(),
          Resolve {
            extensions: string_list(&["8", "..."]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_2(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_3(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_4(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_5(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_6(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "y".into(),
          Resolve {
            extensions: string_list(&["8", "..."]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_7(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_8(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_9(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_10(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );
  }

  #[test]
  fn test_merge_resolver_options_15() {
    let second = || Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "x".into(),
        Resolve {
          extensions: string_list(&["8", "..."]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    };

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_1(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "..."]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_2(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "5"]),
            ..Default::default()
          },
        ),])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_3(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "5", "...", "6"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_4(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "5"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_5(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "5", "...", "6"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_6(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "..."]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_7(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "5"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_8(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "5", "...", "6"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_9(), Resolve::default()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["5"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_10(), Resolve::default()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["5", "...", "6"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      }
    );
  }

  #[test]
  fn test_merge_resolver_options_16() {
    let second = || Resolve {
      extensions: string_list(&["7", "..."]),
      by_dependency: Some(ByDependency::from_iter([
        (
          "x".into(),
          Resolve {
            extensions: string_list(&["8", "..."]),
            ..Default::default()
          },
        ),
        (
          "y".into(),
          Resolve {
            extensions: string_list(&["9", "..."]),
            ..Default::default()
          },
        ),
      ])),
      ..Default::default()
    };

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_1(), second()),
      Resolve {
        extensions: string_list(&["7", "1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_2(), second()),
      Resolve {
        extensions: string_list(&["7", "1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "7", "5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "..."]),
              ..Default::default()
            },
          ),
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_3(), second()),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "default".into(),
            Resolve {
              extensions: string_list(&["7", "..."]),
              ..Default::default()
            },
          ),
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "7", "5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "7", "..."]),
              ..Default::default()
            },
          ),
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_4(), second()),
      Resolve {
        extensions: string_list(&["7", "..."]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "7", "5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_5(), second()),
      Resolve {
        by_dependency: Some(ByDependency::from_iter([
          (
            "default".into(),
            Resolve {
              extensions: string_list(&["7", "..."]),
              ..Default::default()
            },
          ),
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "7", "5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "7", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_6(), second()),
      Resolve {
        extensions: string_list(&["7", "1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "..."]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_7(), second()),
      Resolve {
        extensions: string_list(&["7", "1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "7", "5"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "..."]),
              ..Default::default()
            },
          )
        ])),
        ..Default::default()
      }
    );

    pretty_assertions::assert_eq!(
      merge_resolve(first_case_8(), second()),
      Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "default".into(),
            Resolve {
              extensions: string_list(&["7", "..."]),
              ..Default::default()
            },
          ),
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["8", "7", "5", "...", "6"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["9", "7", "..."]),
              ..Default::default()
            },
          ),
        ])),
        ..Default::default()
      }
    );
  }

  #[test]
  fn test_merge_resolver_options_17() {
    let second = || Resolve {
      extensions: string_list(&["8", "...", "9"]),
      ..Default::default()
    };
    {
      let first = Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["3", "...", "4"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
          (
            "default".into(),
            Resolve {
              extensions: string_list(&["6", "...", "7"]),
              ..Default::default()
            },
          ),
        ])),
        ..Default::default()
      };

      pretty_assertions::assert_eq!(
        merge_resolve(first, second()),
        Resolve {
          extensions: string_list(&["1", "...", "2"]),
          by_dependency: Some(ByDependency::from_iter([
            (
              "x".into(),
              Resolve {
                extensions: string_list(&["8", "3", "...", "4", "9"]),
                ..Default::default()
              },
            ),
            (
              "y".into(),
              Resolve {
                extensions: string_list(&["8", "5", "9"]),
                ..Default::default()
              },
            ),
            (
              "default".into(),
              Resolve {
                extensions: string_list(&["8", "6", "...", "7", "9"]),
                ..Default::default()
              },
            ),
          ])),
          ..Default::default()
        }
      );
    }

    {
      let first = Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              extensions: string_list(&["3", "...", "4"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              extensions: string_list(&["5"]),
              ..Default::default()
            },
          ),
        ])),
        ..Default::default()
      };

      pretty_assertions::assert_eq!(
        merge_resolve(first, second()),
        Resolve {
          extensions: string_list(&["1", "...", "2"]),
          by_dependency: Some(ByDependency::from_iter([
            (
              "x".into(),
              Resolve {
                extensions: string_list(&["8", "3", "...", "4", "9"]),
                ..Default::default()
              },
            ),
            (
              "y".into(),
              Resolve {
                extensions: string_list(&["8", "5", "9"]),
                ..Default::default()
              },
            ),
            (
              "default".into(),
              Resolve {
                extensions: string_list(&["8", "...", "9"]),
                ..Default::default()
              },
            ),
          ])),
          ..Default::default()
        }
      );
    }

    {
      let first = Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "default".into(),
          Resolve {
            extensions: string_list(&["6", "...", "7"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      };

      pretty_assertions::assert_eq!(
        merge_resolve(first, second()),
        Resolve {
          extensions: string_list(&["1", "...", "2"]),
          by_dependency: Some(ByDependency::from_iter([(
            "default".into(),
            Resolve {
              extensions: string_list(&["8", "6", "...", "7", "9"]),
              ..Default::default()
            },
          ),])),
          ..Default::default()
        }
      );
    }

    {
      let first = Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([
          ("x".into(), Resolve::default()),
          ("y".into(), Resolve::default()),
          (
            "default".into(),
            Resolve {
              extensions: string_list(&["6", "...", "7"]),
              ..Default::default()
            },
          ),
        ])),
        ..Default::default()
      };

      pretty_assertions::assert_eq!(
        merge_resolve(first, second()),
        Resolve {
          extensions: string_list(&["1", "...", "2"]),
          by_dependency: Some(ByDependency::from_iter([
            (
              "default".into(),
              Resolve {
                extensions: string_list(&["8", "6", "...", "7", "9"]),
                ..Default::default()
              },
            ),
            (
              "x".into(),
              Resolve {
                extensions: string_list(&["8", "...", "9"]),
                ..Default::default()
              },
            ),
            (
              "y".into(),
              Resolve {
                extensions: string_list(&["8", "...", "9"]),
                ..Default::default()
              },
            ),
          ])),
          ..Default::default()
        }
      );
    }

    {
      let first = Resolve {
        extensions: string_list(&["1", "...", "2"]),
        by_dependency: Some(ByDependency::from_iter([(
          "x".into(),
          Resolve {
            extensions: string_list(&["3", "...", "4"]),
            ..Default::default()
          },
        )])),
        ..Default::default()
      };

      pretty_assertions::assert_eq!(
        merge_resolve(first, second()),
        Resolve {
          extensions: string_list(&["1", "...", "2"]),
          by_dependency: Some(ByDependency::from_iter([
            (
              "x".into(),
              Resolve {
                extensions: string_list(&["8", "3", "...", "4", "9"]),
                ..Default::default()
              },
            ),
            (
              "default".into(),
              Resolve {
                extensions: string_list(&["8", "...", "9"]),
                ..Default::default()
              },
            ),
          ])),
          ..Default::default()
        }
      );
    }
  }

  #[test]
  fn test_merge_resolver_options_18() {
    let first = Resolve {
      extensions: string_list(&["1"]),
      by_dependency: Some(ByDependency::from_iter([
        (
          "x".into(),
          Resolve {
            modules: string_list(&["2"]),
            ..Default::default()
          },
        ),
        (
          "default".into(),
          Resolve {
            main_fields: string_list(&["3"]),
            ..Default::default()
          },
        ),
      ])),
      ..Default::default()
    };

    let second = Resolve {
      by_dependency: Some(ByDependency::from_iter([(
        "y".into(),
        Resolve {
          main_files: string_list(&["4"]),
          ..Default::default()
        },
      )])),
      ..Default::default()
    };

    pretty_assertions::assert_eq!(
      merge_resolve(first, second),
      Resolve {
        extensions: string_list(&["1"]),
        by_dependency: Some(ByDependency::from_iter([
          (
            "x".into(),
            Resolve {
              modules: string_list(&["2"]),
              ..Default::default()
            },
          ),
          (
            "default".into(),
            Resolve {
              main_fields: string_list(&["3"]),
              ..Default::default()
            },
          ),
          (
            "y".into(),
            Resolve {
              main_fields: string_list(&["3"]),
              main_files: string_list(&["4"]),
              ..Default::default()
            },
          ),
        ])),
        ..Default::default()
      }
    )
  }
}
