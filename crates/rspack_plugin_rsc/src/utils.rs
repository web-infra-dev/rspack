use std::{borrow::Cow, collections::VecDeque};

use rspack_collections::Identifiable;
use rspack_core::{Compilation, Module, ModuleType, RscModuleType};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rustc_hash::FxHashSet;
use serde::Serialize;
use urlencoding::encode;

use crate::constants::CSS_REGEX;

pub fn is_federation_virtual_module(module: &dyn Module) -> bool {
  matches!(
    module.module_type(),
    ModuleType::Remote
      | ModuleType::Fallback
      | ModuleType::ProvideShared
      | ModuleType::ConsumeShared
      | ModuleType::ShareContainerShared
      | ModuleType::SelfReference
  )
}

pub fn get_module_resource<'a>(module: &'a dyn Module) -> Cow<'a, str> {
  if let Some(module) = module.as_normal_module() {
    let resource_resolved_data = module.resource_resolved_data();
    let mod_path = resource_resolved_data
      .path()
      .map_or("", |path| path.as_str());
    let mod_query = resource_resolved_data.query().unwrap_or("");
    // We have to always use the resolved request here to make sure the
    // server and client are using the same module path (required by RSC), as
    // the server compiler and client compiler have different resolve configs.
    Cow::Owned(format!("{mod_path}{mod_query}"))
  } else if let Some(module) = module.as_context_module() {
    Cow::Borrowed(module.identifier().as_str())
  } else if is_federation_virtual_module(module) {
    // Federation virtual modules are not normal modules but still need stable,
    // non-empty identities so manifest/resource collection can include shared
    // and remote references.
    Cow::Owned(format!("mf://{}", module.identifier()))
  } else {
    Cow::Borrowed("")
  }
}

pub fn get_canonical_module_resource(compilation: &Compilation, module: &dyn Module) -> String {
  if !is_federation_virtual_module(module) {
    return get_module_resource(module).into_owned();
  }

  const MAX_BFS_DEPTH: usize = 16;
  const MAX_VISITED_MODULES: usize = 256;

  let module_graph = compilation.get_module_graph();
  let mut queue = VecDeque::from([(module.identifier(), 0_usize)]);
  let mut visited: FxHashSet<_> = FxHashSet::default();
  let mut fallback_resource: Option<String> = None;
  visited.insert(module.identifier());

  while let Some((current_identifier, depth)) = queue.pop_front() {
    if depth >= MAX_BFS_DEPTH {
      continue;
    }
    if visited.len() >= MAX_VISITED_MODULES {
      break;
    }

    for dependency_id in module_graph.get_outgoing_deps_in_order(&current_identifier) {
      if visited.len() >= MAX_VISITED_MODULES {
        break;
      }
      let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
        continue;
      };
      let resolved_identifier = connection.resolved_module;
      if !visited.insert(resolved_identifier) {
        continue;
      }

      let Some(resolved_module) = module_graph.module_by_identifier(&resolved_identifier) else {
        continue;
      };
      if !is_federation_virtual_module(resolved_module.as_ref()) {
        let resolved_resource = get_module_resource(resolved_module.as_ref());
        if !resolved_resource.is_empty() {
          let resolved_resource = resolved_resource.into_owned();
          let is_client_module = resolved_module
            .build_info()
            .rsc
            .as_ref()
            .is_some_and(|rsc| rsc.module_type == RscModuleType::Client);
          if is_client_module {
            return resolved_resource;
          }
          fallback_resource.get_or_insert(resolved_resource);
        }
      }

      queue.push_back((resolved_identifier, depth + 1));
    }
  }

  if let Some(resource) = fallback_resource {
    return resource;
  }

  get_module_resource(module).into_owned()
}

/// Extract the shared package request from a federation consume request.
///
/// Examples:
/// - `webpack/sharing/consume/default/react/react` -> `react`
/// - `(server-side-rendering)/webpack/sharing/consume/rsc/rsc-shared/rsc-shared` -> `rsc-shared`
pub fn extract_shared_package_from_consume_request(request: &str) -> Option<String> {
  const CONSUME_SHARED_PREFIX: &str = "webpack/sharing/consume/";

  let request_without_query = request.split('?').next().unwrap_or(request);
  let marker_index = request_without_query.find(CONSUME_SHARED_PREFIX)?;
  let suffix = &request_without_query[marker_index + CONSUME_SHARED_PREFIX.len()..];

  let segments = suffix
    .split('/')
    .filter(|segment| !segment.is_empty())
    .collect::<Vec<_>>();
  if segments.len() < 3 {
    return None;
  }

  // Drop share scope and parse the duplicated `{shareKey}/{request}` tail.
  let tail = &segments[1..];
  if tail.len() >= 2 && tail.len() % 2 == 0 {
    let half = tail.len() / 2;
    if tail[..half] == tail[half..] {
      return Some(tail[..half].join("/"));
    }
  }

  tail.last().map(|segment| (*segment).to_string())
}

pub fn is_css_mod(module: &dyn Module) -> bool {
  if matches!(
    module.module_type(),
    ModuleType::Css | ModuleType::CssModule | ModuleType::CssAuto
  ) {
    return true;
  }
  let resource = get_module_resource(module);
  CSS_REGEX.is_match(resource.as_ref())
}

/// Returns a JSON string literal for `value` (i.e. double-encoded), suitable for embedding into JS.
///
/// Example:
/// - input:  `{"a":1}`
/// - output: "\"{\\\"a\\\":1}\""
pub fn to_json_string_literal<T: ?Sized + Serialize>(value: &T) -> Result<String> {
  serde_json::to_string(&serde_json::to_string(value).to_rspack_result()?).to_rspack_result()
}

pub fn encode_uri_path(file: &str) -> String {
  file
    .split('/')
    .map(|p| encode(p).into_owned())
    .collect::<Vec<_>>()
    .join("/")
}
