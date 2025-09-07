// use rspack_util::fx_hash::{FxHashMap, FxHashSet};
// use serde::Serialize;

// #[derive(Serialize)]
// pub struct DebugJson {
//   pub chunks: FxHashMap<u32, DebugChunkLinkJson>,
//   pub modules: FxHashMap<String, DebugModuleInfo>,
// }

// #[derive(Serialize)]
// pub enum DebugModuleInfo {
//   External(String),
//   ScopeHoisted(DebugScopeHoistedModule),
// }

// #[derive(Serialize)]
// pub struct DebugScopeHoistedModule {
//   pub id: String,
//   pub internal_names: FxHashMap<String, String>,
//   pub exports: FxHashMap<String, String>,
//   pub raw_exports: FxHashMap<String, String>,
//   pub namespace_export_symbol: Option<String>,
//   pub interop_namespace_object_used: bool,
//   pub interop_namespace_object_name: Option<String>,
//   pub interop_namespace_object2_used: bool,
//   pub interop_namespace_object2_name: Option<String>,
//   pub interop_default_access_used: bool,
//   pub interop_default_access_name: Option<String>,
// }

// #[derive(Serialize)]
// pub struct DebugChunkLinkJson {
//   pub exports: FxHashMap<String, FxHashMap<String, String>>,
//   pub imports: FxHashMap<String, FxHashMap<String, String>>,
//   pub required: FxHashSet<String>,
//   pub hoisted_modules: Vec<String>,
//   pub decl_modules: Vec<String>,
// }
