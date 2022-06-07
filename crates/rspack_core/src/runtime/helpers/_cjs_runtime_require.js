function __rspack_require__(id) {
  const self = this;
  let mod = globalThis.rs.m[id];
  if (!mod) {
    throw new Error(id + ' not exits');
  }
  if (mod.loaded) {
    return mod.exports;
  }
  if (self instanceof globalThis.rs.Module) {
    this.children.add(mod);
    mod.parents.add(this);
  }
  mod.loaded = true;
  mod.factory(__rspack_require__.bind(mod), mod, mod.exports);
  return mod.exports;
}

globalThis.rs.require = globalThis.rs.require || __rspack_require__;
