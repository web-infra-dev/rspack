// rs cjs runtime bootstrap
globalThis.rs = globalThis.rs || {};

var __rspack_modules__ = {};
globalThis.rs.m = globalThis.rs.m || __rspack_modules__;

function __rspack_define__(id, factory) {
  var mod = __rspack_modules__[id];
  if (!mod) {
    __rspack_modules__[id] = new Module({
      factory: factory,
      loaded: false,
      exports: {},
      id,
    });
  } else {
    console.debug('repeated define for', id);
    // mod.loaded = false;
    // mod.exports = {};
    // mod.factory = factory;
  }
}
globalThis.rs.define = globalThis.rs.define || __rspack_define__;

function Module(options) {
  this.id = options.id;
  this.factory = options.factory;
  this.loaded = options.loaded;
  this.exports = options.exports;
  this.children = new Set(); // children module
  this.parents = new Set(); //  parent module
}

globalThis.rs.Module = globalThis.rs.Module || Module;
