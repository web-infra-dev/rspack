(function () {
  let modules = {};
  class Hot {
    constructor(id) {
      this.id = id;
      this.accepts = [];
    }
    accept(ids, callback) {
      if (ids === undefined) {
        this.accepts.push({
          ids: this.id,
          accept: undefined,
        });
      } else if (typeof ids === 'function') {
        this.accepts.push({
          ids: this.id,
          accept: ids,
        });
      } else {
        this.accepts.push({
          ids,
          accept: callback,
        });
      }
    }
    dispose(callback) {
      this.accepts.push({
        id: this.id,
        dispose: callback,
      });
    }
  }
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
  function define(id, factory) {
    const mod = modules[id];
    if (!mod) {
      modules[id] = new Module({
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
  async function dynamic_require(module_id, chunk_id) {
    /**
     * todo  get chunkd_id from module_id
     */
    if (chunk_id) {
      await import('http://127.0.01:4444/' + chunk_id);
    }
    const result = require(module_id);
    return result;
  }
  function require(id) {
    const self = this;
    let mod = modules[id];
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
    mod.hot = new Hot(id);
    mod.loaded = true;
    mod.factory(require.bind(mod), mod, mod.exports);
    return mod.exports;
  }

  globalThis.rs = {
    define: define,
    require: require,
    dynamic_require: dynamic_require,
    m: modules,
  };
})();
