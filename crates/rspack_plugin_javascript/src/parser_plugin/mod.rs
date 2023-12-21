mod drive;
mod require_context_dependency_parser_plugin;
mod r#trait;

pub use self::drive::JavaScriptParserPluginDrive;
pub use self::r#trait::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
pub use self::require_context_dependency_parser_plugin::RequireContextDependencyParserPlugin;
