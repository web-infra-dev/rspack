// get default export and interop with esm
__get_default_export__ = (module) => {
  return module && module.__esModule ? module['default'] : () => module
}

globalThis.rs.getDefaultExport = globalThis.rs.getDefaultExport || __get_default_export__
