//! Symbol registry keys. Constant string references have a static lifetime.
//! Keep shared keys in sync with packages/rspack/src/util/symbols.ts.

pub(crate) const MODULE_IDENTIFIER_SYMBOL: &str = "rspack.module.identifier";
pub(crate) const MODULE_BUILD_INFO_SYMBOL: &str = "rspack.module.buildInfo";
pub(crate) const BUILD_INFO_ASSETS_SYMBOL: &str = "rspack.buildInfo.assets";
pub(crate) const BUILD_INFO_FILE_DEPENDENCIES_SYMBOL: &str = "rspack.buildInfo.fileDependencies";
pub(crate) const BUILD_INFO_CONTEXT_DEPENDENCIES_SYMBOL: &str =
  "rspack.buildInfo.contextDependencies";
pub(crate) const BUILD_INFO_MISSING_DEPENDENCIES_SYMBOL: &str =
  "rspack.buildInfo.missingDependencies";
pub(crate) const BUILD_INFO_BUILD_DEPENDENCIES_SYMBOL: &str = "rspack.buildInfo.buildDependencies";
pub(crate) const COMMIT_CUSTOM_FIELDS_SYMBOL: &str = "rspack.buildInfo.commitCustomFields";
pub(crate) const RUST_ERROR_SYMBOL: &str = "rspack.error.rust";
pub(crate) const CIRCULAR_CONNECTION_SYMBOL: &str =
  "rspack.moduleGraphConnection.circularConnection";
pub(crate) const TRANSITIVE_ONLY_SYMBOL: &str = "rspack.moduleGraphConnection.transitiveOnly";
pub(crate) const JS_HELPERS_SYMBOL: &str = "rspack.jsHelpers";
pub(crate) const FILE_SYSTEM_DEPENDENCY_STRING_POOL_SYMBOL: &str =
  "rspack.fileSystemDependencyStringPool";
pub(crate) const FILE_SYSTEM_DEPENDENCY_ARRAY_SYMBOL: &str = "rspack.fileSystemDependencyArray";
