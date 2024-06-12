// should not panic at crates/rspack_plugin_javascript/src/dependency/context/require_context_dependency.rs

require.context("./test", false, /\.test\.js$/);

