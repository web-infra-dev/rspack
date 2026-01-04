use std::borrow::Cow;

use futures::future::BoxFuture;
use rspack_collections::Identifiable;
use rspack_core::{
  ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation, CompilerId,
  ConcatenatedInnerModule, Module, ModuleGraphRef, ModuleId, ModuleIdentifier, ModuleType,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use serde::Serialize;
use urlencoding::encode;

use crate::constants::CSS_REGEX;

pub fn get_module_resource<'a>(module: &'a dyn Module) -> Cow<'a, str> {
  if let Some(module) = module.as_normal_module() {
    let resource_resolved_data = module.resource_resolved_data();
    let mod_path = resource_resolved_data
      .path()
      .map(|path| path.as_str())
      .unwrap_or("");
    let mod_query = resource_resolved_data.query().unwrap_or("");
    // We have to always use the resolved request here to make sure the
    // server and client are using the same module path (required by RSC), as
    // the server compiler and client compiler have different resolve configs.
    Cow::Owned(format!("{}{}", mod_path, mod_query))
  } else if let Some(module) = module.as_context_module() {
    Cow::Borrowed(module.identifier().as_str())
  } else {
    Cow::Borrowed("")
  }
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

pub struct ChunkModules<'a> {
  compilation: &'a Compilation,
  module_graph: &'a ModuleGraphRef<'a>,
  chunk_groups_iter: Box<dyn Iterator<Item = (&'a ChunkGroupUkey, &'a ChunkGroup)> + 'a>,
  chunks_iter: Option<std::slice::Iter<'a, ChunkUkey>>,
  modules_iter: Option<std::collections::hash_set::Iter<'a, ModuleIdentifier>>,
  concatenated_modules_iter: Option<std::slice::Iter<'a, ConcatenatedInnerModule>>,
  current_chunk: Option<ChunkUkey>,
  current_chunk_group: Option<&'a ChunkGroup>,
}

impl<'a> ChunkModules<'a> {
  pub fn new(compilation: &'a Compilation, module_graph: &'a ModuleGraphRef) -> Self {
    let chunk_groups_iter = Box::new(compilation.chunk_group_by_ukey.iter());
    Self {
      compilation,
      module_graph,
      chunk_groups_iter,
      chunks_iter: None,
      modules_iter: None,
      concatenated_modules_iter: None,
      current_chunk: None,
      current_chunk_group: None,
    }
  }
}

impl<'a> Iterator for ChunkModules<'a> {
  type Item = (ModuleIdentifier, ModuleId);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(concatenated_modules_iter) = self.concatenated_modules_iter.as_mut() {
        if let Some(module) = concatenated_modules_iter.next() {
          match ChunkGraph::get_module_id(&self.compilation.module_ids_artifact, module.id) {
            Some(module_id) => {
              return Some((module.id, module_id.clone()));
            }
            None => {
              continue;
            }
          }
        } else {
          self.concatenated_modules_iter = None;
        }
      }

      if let Some(modules_iter) = self.modules_iter.as_mut() {
        if let Some(module_identifier) = modules_iter.next() {
          match ChunkGraph::get_module_id(&self.compilation.module_ids_artifact, *module_identifier)
          {
            Some(module_id) => {
              return Some((*module_identifier, module_id.clone()));
            }
            None => {
              let Some(module) = self.module_graph.module_by_identifier(module_identifier) else {
                continue;
              };
              let Some(concatenated_module) = module.as_concatenated_module() else {
                continue;
              };
              let concatenated_modules = concatenated_module.get_modules();
              if !concatenated_modules.is_empty() {
                self.concatenated_modules_iter = Some(concatenated_module.get_modules().iter());
                continue;
              }
              continue;
            }
          }
        } else {
          self.modules_iter = None;
        }
      }

      if let Some(ref mut chunks_iter) = self.chunks_iter {
        if let Some(chunk_ukey) = chunks_iter.next() {
          self.current_chunk = Some(*chunk_ukey);

          let chunk_modules = self
            .compilation
            .chunk_graph
            .get_chunk_modules_identifier(chunk_ukey);

          if !chunk_modules.is_empty() {
            self.modules_iter = Some(chunk_modules.iter());
            continue;
          }
          continue;
        } else {
          self.chunks_iter = None;
          self.current_chunk = None;
          self.current_chunk_group = None;
        }
      }

      if let Some((_, chunk_group)) = self.chunk_groups_iter.next() {
        self.current_chunk_group = Some(chunk_group);
        if !chunk_group.chunks.is_empty() {
          self.chunks_iter = Some(chunk_group.chunks.iter());
          continue;
        }
        continue;
      }

      return None;
    }
  }
}

pub type GetServerCompilerId =
  Box<dyn Fn() -> BoxFuture<'static, Result<CompilerId>> + Sync + Send>;

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
