// Keep these registry keys in sync with crates/rspack_binding_api/src/symbols.rs.
export const MODULE_IDENTIFIER_SYMBOL = Symbol.for('rspack.module.identifier');
export const MODULE_BUILD_INFO_SYMBOL = Symbol.for('rspack.module.buildInfo');
export const COMPILATION_HOOKS_MAP_SYMBOL = Symbol.for(
  'rspack.compilation.hooksMap',
);
export const BUILD_INFO_ASSETS_SYMBOL = Symbol.for('rspack.buildInfo.assets');
export const BUILD_INFO_FILE_DEPENDENCIES_SYMBOL = Symbol.for(
  'rspack.buildInfo.fileDependencies',
);
export const BUILD_INFO_CONTEXT_DEPENDENCIES_SYMBOL = Symbol.for(
  'rspack.buildInfo.contextDependencies',
);
export const BUILD_INFO_MISSING_DEPENDENCIES_SYMBOL = Symbol.for(
  'rspack.buildInfo.missingDependencies',
);
export const BUILD_INFO_BUILD_DEPENDENCIES_SYMBOL = Symbol.for(
  'rspack.buildInfo.buildDependencies',
);
export const COMMIT_CUSTOM_FIELDS_SYMBOL = Symbol.for(
  'rspack.buildInfo.commitCustomFields',
);
export const CIRCULAR_CONNECTION_SYMBOL = Symbol.for(
  'rspack.moduleGraphConnection.circularConnection',
);
export const TRANSITIVE_ONLY_SYMBOL = Symbol.for(
  'rspack.moduleGraphConnection.transitiveOnly',
);
