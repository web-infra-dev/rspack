mod amd;
mod api_plugin;
mod check_var_decl;
mod common_js_exports_parse_plugin;
mod common_js_imports_parse_plugin;
mod common_js_plugin;
mod compatibility_plugin;
mod r#const;
mod drive;
mod esm_detection_parser_plugin;
mod esm_export_dependency_parser_plugin;
mod esm_import_dependency_parser_plugin;
mod esm_top_level_this_plugin;
mod exports_info_api_plugin;
mod import_meta_context_dependency_parser_plugin;
mod import_meta_plugin;
mod import_parser_plugin;
mod initialize_evaluating;
mod inline_const;
mod inner_graph;
mod javascript_meta_info_plugin;
pub mod node_stuff_plugin;
mod override_strict_plugin;
mod require_context_dependency_parser_plugin;
mod require_ensure_dependencies_block_parse_plugin;
mod r#trait;
mod url_plugin;
mod use_strict_plugin;
mod webpack_included_plugin;
mod worker_plugin;

pub mod define_plugin;
pub mod hot_module_replacement_plugin;
pub mod provide_plugin;

pub use self::r#trait::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
pub(crate) use self::{
  amd::{
    AMDDefineDependencyParserPlugin, AMDParserPlugin, AMDRequireDependenciesBlockParserPlugin,
  },
  api_plugin::APIPlugin,
  check_var_decl::CheckVarDeclaratorIdent,
  common_js_exports_parse_plugin::CommonJsExportsParserPlugin,
  common_js_imports_parse_plugin::CommonJsImportsParserPlugin,
  common_js_plugin::CommonJsPlugin,
  compatibility_plugin::CompatibilityPlugin,
  r#const::{ConstPlugin, is_logic_op},
  drive::JavaScriptParserPluginDrive,
  esm_detection_parser_plugin::ESMDetectionParserPlugin,
  esm_export_dependency_parser_plugin::ESMExportDependencyParserPlugin,
  esm_import_dependency_parser_plugin::ESMImportDependencyParserPlugin,
  esm_top_level_this_plugin::ESMTopLevelThisParserPlugin,
  exports_info_api_plugin::ExportsInfoApiPlugin,
  import_meta_context_dependency_parser_plugin::ImportMetaContextDependencyParserPlugin,
  import_meta_plugin::{ImportMetaDisabledPlugin, ImportMetaPlugin},
  import_parser_plugin::{ImportParserPlugin, ImportsReferencesState},
  initialize_evaluating::InitializeEvaluating,
  inline_const::{
    InlineConstPlugin, connection_active_inline_value_for_esm_export_imported_specifier,
    connection_active_inline_value_for_esm_import_specifier, is_export_inlined,
  },
  inner_graph::{connection_active_used_by_exports, plugin::*, state::InnerGraphState},
  javascript_meta_info_plugin::JavascriptMetaInfoPlugin,
  node_stuff_plugin::NodeStuffPlugin,
  override_strict_plugin::OverrideStrictPlugin,
  require_context_dependency_parser_plugin::RequireContextDependencyParserPlugin,
  require_ensure_dependencies_block_parse_plugin::RequireEnsureDependenciesBlockParserPlugin,
  url_plugin::URLPlugin,
  use_strict_plugin::UseStrictPlugin,
  webpack_included_plugin::WebpackIsIncludedPlugin,
  worker_plugin::WorkerPlugin,
};

pub static JS_DEFAULT_KEYWORD: std::sync::LazyLock<swc_core::atoms::Atom> =
  std::sync::LazyLock::new(|| swc_core::atoms::atom!("default"));

pub static DEFAULT_STAR_JS_WORD: std::sync::LazyLock<swc_core::atoms::Atom> =
  std::sync::LazyLock::new(|| swc_core::atoms::atom!("*default*"));
