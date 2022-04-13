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
class Module {
  constructor(options){
    this.factory = options.factory;
    this.loaded = options.loaded;
    this.exports = options.exports;
    this.children = new Set(); // children module
    this.parents = new Set(); //  parent module
  }
}
function define(id, factory) {
  modules[id] = new Module({ factory: factory, loaded: false, exports: {} });
}
function require(id) {
  const self = this;
  let realId = id;
  let mod = modules[realId];
  if (!mod) {
    console.error(realId + 'not exists');
  }
  if (mod.loaded) {
    return mod.exports;
  }
  if(self instanceof Module){
    this.children.add(mod);
    mod.parents.add(this)
  }
  mod.hot = new Hot(id);
  mod.factory(require.bind(mod), mod.exports,mod);
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
