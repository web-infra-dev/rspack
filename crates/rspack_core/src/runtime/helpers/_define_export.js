function __define_export__(exports, definition) {
  for (var key in definition) {
    if (rs.has_own_property(definition, key) && !rs.has_own_property(exports, key)) {
      Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
    }
  }
}

globalThis.rs.define_export = globalThis.rs.define_export || __define_export__;
