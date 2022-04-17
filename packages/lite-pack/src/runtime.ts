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
          ids: this.id,
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
    this.id = options.id
    this.factory = options.factory;
    this.loaded = options.loaded;
    this.exports = options.exports;
    this.children = new Set(); // children module
    this.parents = new Set(); //  parent module
  }
}
function define(id, factory) {
  const mod = modules[id];
  if(!mod){
    modules[id] = new Module({ factory: factory, loaded: false, exports: {}, id });
  }else {
    mod.loaded = false;
    mod.exports = {};
    mod.factory =factory;
  }
  
}
async function dynamic_require(id,chunkId){
  await import("http://localhost:4444/"+chunkId);
  const result = require(id);
  return result;
}
function require(id) {
  const self = this;
  let mod = modules[id];
  if (!mod) {
    throw new Error(id+" not exits");
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
  dynamic_require: dynamic_require,
  m: modules
}})();
    `;
  }
}
