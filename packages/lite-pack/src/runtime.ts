export class Runtime {
  render() {
    return `
(function(){
let __modules = {};
function define(id, factory) {
  if (!__modules[id]) {
    __modules[id] = { factory: factory, loaded: false, exports: {} };
  } else {
    console.error('define' + id + 'twice');
  }
}
function require(id) {
  let realId = id;
  let mod = __modules[realId];
  if (!mod) {
    console.error(realId + 'not exists');
  }
  if (mod.loaded) {
    return mod.exports;
  }
  mod.factory(require, mod.exports);
  mod.loaded = true;
  return mod.exports;
}
globalThis.rs = {
  define:define,
  require:require
}
  })();
    `;
  }
}
