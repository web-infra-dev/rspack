use std::{cell::RefCell, collections::HashMap, path::Path};

use rspack_core::StatsModule;

use super::data::{StatsExpose, StatsShared};

const HOT_UPDATE_SUFFIX: &str = ".hot-update";

pub fn compose_id_with_separator(container: &str, name: &str) -> String {
  format!("{container}:{name}")
}

pub fn is_hot_file(file: &str) -> bool {
  file.contains(HOT_UPDATE_SUFFIX)
}

pub fn strip_ext(path: &str) -> String {
  match Path::new(path).extension() {
    Some(_) => path
      .trim_end_matches(
        Path::new(path)
          .extension()
          .and_then(|e| e.to_str())
          .map(|e| format!(".{e}"))
          .unwrap_or_default()
          .as_str(),
      )
      .to_string(),
    None => path.to_string(),
  }
}

pub fn parse_container_exposes_from_identifier(
  identifier: &str,
) -> Option<Vec<(String, Option<String>, String)>> {
  let start = identifier.find('[')?;
  let slice = &identifier[start..];
  let mut depth = 0_usize;
  let mut end_idx = None;
  for (i, ch) in slice.char_indices() {
    match ch {
      '[' => depth += 1,
      ']' => {
        depth -= 1;
        if depth == 0 {
          end_idx = Some(i);
          break;
        }
      }
      _ => {}
    }
  }
  let end = end_idx?;
  let json_str = &slice[..=end];
  let val: serde_json::Value = serde_json::from_str(json_str).ok()?;
  let arr = val.as_array()?;
  let mut ret: Vec<(String, Option<String>, String)> = Vec::new();
  for item in arr {
    if let Some(tuple) = item.as_array() {
      if tuple.len() == 2 {
        let expose_key = tuple[0].as_str().unwrap_or_default().to_string();
        if let Some(obj) = tuple[1].as_object() {
          let import = obj
            .get("import")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
          let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
          ret.push((expose_key, name, import));
        }
      }
    }
  }
  Some(ret)
}

pub fn ensure_shared_entry<'a>(
  shared_map: &'a mut HashMap<String, StatsShared>,
  container_name: &str,
  pkg: &str,
) -> &'a mut StatsShared {
  shared_map
    .entry(pkg.to_string())
    .or_insert_with(|| StatsShared {
      id: compose_id_with_separator(container_name, pkg),
      name: pkg.to_string(),
      version: String::new(),
      requiredVersion: None,
      singleton: None,
      hash: None,
      assets: super::data::StatsAssetsGroup::default(),
      usedIn: Vec::new(),
    })
}

pub fn record_shared_usage<'a>(
  shared_usage_links: &RefCell<Vec<(String, String)>>,
  pkg: &str,
  module: &StatsModule<'a>,
) {
  if let Some(issuer_name) = module.issuer_name.as_ref() {
    let key = strip_ext(issuer_name);
    shared_usage_links.borrow_mut().push((pkg.to_string(), key));
  }
  if let Some(reasons) = module.reasons.as_ref() {
    for reason in reasons {
      if let Some(module_name) = reason.user_request {
        let key = strip_ext(module_name);
        shared_usage_links.borrow_mut().push((pkg.to_string(), key));
      }
    }
  }
}

pub fn parse_provide_shared_identifier(identifier: &str) -> Option<(String, String)> {
  let (before_request, _) = identifier.split_once(" = ")?;
  let token = before_request.split_whitespace().last()?;
  let (name, version) = token.split_once('@')?;
  Some((name.to_string(), version.to_string()))
}

pub fn parse_consume_shared_identifier(identifier: &str) -> Option<(String, Option<String>)> {
  let (_, rest) = identifier.split_once(") ")?;
  let token = rest.split_whitespace().next()?;
  let (name, version) = token.split_once('@')?;
  let version = version.trim();
  let required = if version.is_empty() || version == "*" {
    None
  } else {
    Some(version.to_string())
  };
  Some((name.to_string(), required))
}

pub fn collect_expose_requirements(
  shared_map: &mut HashMap<String, StatsShared>,
  exposes_map: &mut HashMap<String, StatsExpose>,
  links: Vec<(String, String)>,
) {
  for (pkg, expose_key) in links {
    if let Some(expose) = exposes_map.get_mut(&expose_key) {
      if !expose.requires.contains(&pkg) {
        expose.requires.push(pkg.clone());
      }
      if let Some(shared) = shared_map.get_mut(&pkg) {
        shared.usedIn.push(expose.path.clone());
      }
    }
  }
}
