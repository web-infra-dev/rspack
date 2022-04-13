export class Runtime {
  render() {
    return /*javascript*/ `
(function(){
let modules = {};
  class Hot {
    constructor(id) {
      this.id = id;
      this.accepts = [];
    }
    accept(ids, callback) {
      if (typeof ids === "function") {
        this.accepts.push({
          id: this.id,
          accept: ids
        });
      } else {
        this.accepts.push({
          ids,
          accept: callback
        });
      }
    }
    dispose(callback) {
      this.accepts.push({
        id: this.id,
        dispose: callback
      });
    }
  };
function define(id, factory) {
  modules[id] = { factory: factory, loaded: false, exports: {} };
}
function require(id) {
  let realId = id;
  let mod = modules[realId];
  if (!mod) {
    console.error(realId + 'not exists');
  }
  if (mod.loaded) {
    return mod.exports;
  }
  mod.hot = new Hot(id);
  mod.factory(require, mod.exports,mod);
  mod.loaded = true;
  return mod.exports;
}
globalThis.rs = {
  define:define,
  require:require,
  m: modules
}})();
    `;
  }
}
