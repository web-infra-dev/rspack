mod amd;
mod api_plugin;
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
mod import_phase;
mod initialize_evaluating;
mod inline_const;
pub(crate) mod inner_graph;
mod is_included_plugin;
mod javascript_meta_info_plugin;
pub mod node_stuff_plugin;
mod override_strict_plugin;
mod require_context_dependency_parser_plugin;
mod require_ensure_dependencies_block_parse_plugin;
mod r#trait;
mod url_plugin;
mod use_strict_plugin;
mod worker_plugin;

pub mod define_plugin;
pub mod hot_module_replacement_plugin;
pub mod provide_plugin;
pub mod side_effects_parser_plugin;

pub(crate) use self::{
  amd::{
    AMDDefineDependencyParserPlugin, AMDParserPlugin, AMDRequireDependenciesBlockParserPlugin,
  },
  api_plugin::APIPlugin,
  common_js_exports_parse_plugin::CommonJsExportsParserPlugin,
  common_js_imports_parse_plugin::{
    CREATE_REQUIRE_EVALUATED_TAG, CREATE_REQUIRE_SPECIFIER_TAG, CREATED_REQUIRE_IDENTIFIER_TAG,
    CommonJsImportsParserPlugin, CreatedRequireTagData, RequireReferencesState,
    evaluate_create_require_new_expression, is_create_require_namespace_member,
    is_create_require_specifier,
  },
  common_js_plugin::CommonJsPlugin,
  compatibility_plugin::CompatibilityPlugin,
  r#const::ConstPlugin,
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
  inline_const::{ConstValue, ConstValuePlugin},
  inner_graph::{connection_active_used_by_exports, plugin::*, runtime_condition_used_by_exports},
  is_included_plugin::IsIncludedPlugin,
  javascript_meta_info_plugin::JavascriptMetaInfoPlugin,
  node_stuff_plugin::NodeStuffPlugin,
  override_strict_plugin::OverrideStrictPlugin,
  require_context_dependency_parser_plugin::RequireContextDependencyParserPlugin,
  require_ensure_dependencies_block_parse_plugin::RequireEnsureDependenciesBlockParserPlugin,
  side_effects_parser_plugin::SideEffectsParserPlugin,
  url_plugin::{URLPlugin, get_url_request},
  use_strict_plugin::UseStrictPlugin,
  worker_plugin::WorkerPlugin,
};
pub use self::{
  inner_graph::{deferred_pure_check_is_impure, has_impure_deferred_pure_checks},
  r#trait::{
    BoxJavascriptParserPlugin, JavascriptParserPlugin, JavascriptParserPluginHook,
    JavascriptParserPluginHooks,
  },
};

pub static JS_DEFAULT_KEYWORD: std::sync::LazyLock<swc_atoms::Atom> =
  std::sync::LazyLock::new(|| swc_atoms::atom!("default"));

pub static DEFAULT_STAR_JS_WORD: std::sync::LazyLock<swc_atoms::Atom> =
  std::sync::LazyLock::new(|| swc_atoms::atom!("*default*"));
