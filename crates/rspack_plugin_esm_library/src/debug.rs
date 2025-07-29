use rspack_collections::{IdentifierIndexMap, UkeyMap};
use rspack_core::{Compilation, ModuleInfo};
use rspack_util::fx_hash::{FxHashMap, FxHashSet};
use serde::Serialize;

#[derive(Serialize)]
pub struct DebugJson {
  pub chunks: FxHashMap<u32, DebugChunkLinkJson>,
  pub modules: FxHashMap<String, DebugModuleInfo>,
}

#[derive(Serialize)]
pub enum DebugModuleInfo {
  External(String),
  ScopeHoisted(DebugScopeHoistedModule),
}

#[derive(Serialize)]
pub struct DebugScopeHoistedModule {
  pub id: String,
  pub internal_names: FxHashMap<String, String>,
  pub exports: FxHashMap<String, String>,
  pub raw_exports: FxHashMap<String, String>,
  pub namespace_export_symbol: Option<String>,
  pub interop_namespace_object_used: bool,
  pub interop_namespace_object_name: Option<String>,
  pub interop_namespace_object2_used: bool,
  pub interop_namespace_object2_name: Option<String>,
  pub interop_default_access_used: bool,
  pub interop_default_access_name: Option<String>,
}

#[derive(Serialize)]
pub struct DebugChunkLinkJson {
  pub exports: FxHashMap<String, FxHashSet<String>>,
  pub static_exports: Vec<String>,
  pub imports: FxHashMap<String, FxHashMap<String, String>>,
  pub required: FxHashSet<String>,
  pub hoisted_modules: Vec<String>,
  pub decl_modules: Vec<String>,
}

pub fn get_debug_info(
  compilation: &Compilation,
  modules_map: &IdentifierIndexMap<ModuleInfo>,
) -> String {
  let chunks = compilation
    .chunk_graph
    .link
    .as_ref()
    .expect("should have link data")
    .clone();

  let debug_json = DebugJson {
    chunks: chunks
      .into_iter()
      .map(|(chunk_ukey, context)| {
        (
          chunk_ukey.as_u32(),
          DebugChunkLinkJson {
            exports: context
              .exports
              .into_iter()
              .map(|(id, exports)| {
                (
                  id.to_string(),
                  exports.into_iter().map(|atom| atom.to_string()).collect(),
                )
              })
              .collect(),
            static_exports: context
              .static_exports
              .into_iter()
              .map(|atom| atom.to_string())
              .collect(),
            imports: context
              .imports
              .into_iter()
              .map(|(id, imports)| {
                (
                  id.to_string(),
                  imports
                    .into_iter()
                    .map(|(atom, symbol)| (atom.to_string(), symbol.to_string()))
                    .collect(),
                )
              })
              .collect(),
            required: context
              .required
              .into_iter()
              .map(|(id, _)| id.to_string())
              .collect(),
            hoisted_modules: context
              .hoisted_modules
              .into_iter()
              .map(|id| id.to_string())
              .collect(),
            decl_modules: context
              .decl_modules
              .into_iter()
              .map(|id| id.to_string())
              .collect(),
          },
        )
      })
      .collect(),
    modules: modules_map
      .iter()
      .map(|(module_id, info)| {
        (
          module_id.to_string(),
          match info {
            ModuleInfo::External(external_info) => {
              DebugModuleInfo::External(external_info.module.to_string())
            }
            ModuleInfo::Concatenated(concate_info) => {
              DebugModuleInfo::ScopeHoisted(DebugScopeHoistedModule {
                id: concate_info.module.to_string(),
                internal_names: concate_info
                  .internal_names
                  .iter()
                  .map(|(orig, internal)| (orig.to_string(), internal.to_string()))
                  .collect(),
                exports: concate_info
                  .export_map
                  .clone()
                  .unwrap_or_default()
                  .iter()
                  .map(|(export, internal)| (export.to_string(), internal.to_string()))
                  .collect(),
                raw_exports: concate_info
                  .raw_export_map
                  .clone()
                  .unwrap_or_default()
                  .iter()
                  .map(|(export, internal)| (export.to_string(), internal.to_string()))
                  .collect(),
                namespace_export_symbol: concate_info
                  .namespace_export_symbol
                  .clone()
                  .map(|atom| atom.to_string()),
                interop_namespace_object_used: concate_info.interop_namespace_object_used,
                interop_namespace_object_name: concate_info
                  .interop_namespace_object_name
                  .clone()
                  .map(|atom| atom.to_string()),
                interop_namespace_object2_used: concate_info.interop_namespace_object2_used,
                interop_namespace_object2_name: concate_info
                  .interop_namespace_object2_name
                  .clone()
                  .map(|atom| atom.to_string()),
                interop_default_access_used: concate_info.interop_default_access_used,
                interop_default_access_name: concate_info
                  .interop_default_access_name
                  .clone()
                  .map(|atom| atom.to_string()),
              })
            }
          },
        )
      })
      .collect(),
  };

  serde_json::to_string(&debug_json).expect("should serialize debug json")
}
