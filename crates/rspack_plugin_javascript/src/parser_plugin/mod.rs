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
mod inner_graph;
mod javascript_meta_info_plugin;
mod node_stuff_plugin;
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

pub(crate) use self::api_plugin::APIPlugin;
pub(crate) use self::check_var_decl::CheckVarDeclaratorIdent;
pub(crate) use self::common_js_exports_parse_plugin::CommonJsExportsParserPlugin;
pub(crate) use self::common_js_imports_parse_plugin::CommonJsImportsParserPlugin;
pub(crate) use self::common_js_plugin::CommonJsPlugin;
pub(crate) use self::compatibility_plugin::CompatibilityPlugin;
pub(crate) use self::drive::JavaScriptParserPluginDrive;
pub(crate) use self::esm_detection_parser_plugin::ESMDetectionParserPlugin;
pub(crate) use self::esm_export_dependency_parser_plugin::ESMExportDependencyParserPlugin;
pub(crate) use self::esm_import_dependency_parser_plugin::ESMImportDependencyParserPlugin;
pub(crate) use self::esm_top_level_this_plugin::ESMTopLevelThisParserPlugin;
pub(crate) use self::exports_info_api_plugin::ExportsInfoApiPlugin;
pub(crate) use self::import_meta_context_dependency_parser_plugin::ImportMetaContextDependencyParserPlugin;
pub(crate) use self::import_meta_plugin::{ImportMetaDisabledPlugin, ImportMetaPlugin};
pub(crate) use self::import_parser_plugin::ImportParserPlugin;
pub(crate) use self::initialize_evaluating::InitializeEvaluating;
pub(crate) use self::inner_graph::{plugin::*, state::InnerGraphState};
pub(crate) use self::javascript_meta_info_plugin::JavascriptMetaInfoPlugin;
pub(crate) use self::node_stuff_plugin::NodeStuffPlugin;
pub(crate) use self::override_strict_plugin::OverrideStrictPlugin;
pub(crate) use self::r#const::{is_logic_op, ConstPlugin};
pub use self::r#trait::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
pub(crate) use self::require_context_dependency_parser_plugin::RequireContextDependencyParserPlugin;
pub(crate) use self::require_ensure_dependencies_block_parse_plugin::RequireEnsureDependenciesBlockParserPlugin;
pub(crate) use self::url_plugin::URLPlugin;
pub(crate) use self::use_strict_plugin::UseStrictPlugin;
pub(crate) use self::webpack_included_plugin::WebpackIsIncludedPlugin;
pub(crate) use self::worker_plugin::WorkerPlugin;

pub static JS_DEFAULT_KEYWORD: std::sync::LazyLock<swc_core::atoms::Atom> =
  std::sync::LazyLock::new(|| swc_core::atoms::atom!("default"));

pub static DEFAULT_STAR_JS_WORD: std::sync::LazyLock<swc_core::atoms::Atom> =
  std::sync::LazyLock::new(|| swc_core::atoms::atom!("*default*"));
