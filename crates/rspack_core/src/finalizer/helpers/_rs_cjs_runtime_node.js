(function () {
  let __rspack_modules__ = {};
  // class Hot {
  //   constructor(id) {
  //     this.id = id;
  //     this.accepts = [];
  //   }
  //   accept(ids, callback) {
  //     if (ids === undefined) {
  //       this.accepts.push({
  //         ids: this.id,
  //         accept: undefined,
  //       });
  //     } else if (typeof ids === 'function') {
  //       this.accepts.push({
  //         ids: this.id,
  //         accept: ids,
  //       });
  //     } else {
  //       this.accepts.push({
  //         ids,
  //         accept: callback,
  //       });
  //     }
  //   }
  //   dispose(callback) {
  //     this.accepts.push({
  //       id: this.id,
  //       dispose: callback,
  //     });
  //   }
  // }
  class Module {
    constructor(options) {
      this.id = options.id;
      this.factory = options.factory;
      this.loaded = options.loaded;
      this.exports = options.exports;
      this.children = new Set(); // children module
      this.parents = new Set(); //  parent module
    }
  }
  function __rspack_define__(id, factory) {
    const mod = __rspack_modules__[id];
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
  async function __rspack_dynamic_require__(module_id, chunk_id) {
    return import(`./${chunk_id}`);
  }
  function __rspack_require__(id) {
    const self = this;
    let mod = __rspack_modules__[id];
    if (!mod) {
      throw new Error(id + ' not exits');
    }
    if (mod.loaded) {
      return mod.exports;
    }
    if (self instanceof Module) {
      this.children.add(mod);
      mod.parents.add(this);
    }
    mod.loaded = true;
    mod.factory(__rspack_require__.bind(mod), mod, mod.exports);
    return mod.exports;
  }

  globalThis.rs = globalThis.rs || {
    define: __rspack_define__,
    require: __rspack_require__,
    dynamic_require: __rspack_dynamic_require__,
    m: __rspack_modules__,
  };
})();
