(function () { // runtime instance
var runtime = new Object();
self['__rspack_runtime__'] = runtime;// mount Modules
(function () {
  runtime.installedModules = {/* __INSTALLED_MODULES__*/};
})();

// mount Chunks
(function () {
  runtime.installedChunks = {};
})();

// mount ModuleCache
(function () {
  runtime.moduleCache = {};
})();(function () {
  runtime.checkById = function (obj, prop) {
    return Object.prototype.hasOwnProperty.call(obj, prop);
  };
})();// mount PublicPath
(function () {
  runtime.publicPath = "/";
})();// The require function
function __rspack_require__(moduleId) {
  var cachedModule = this.moduleCache[moduleId];
  if (cachedModule !== undefined) {
    return cachedModule.exports;
  }

  // Create a new module (and put it into the cache)
  var module = (this.moduleCache[moduleId] = {
    // no module.id needed
    // no module.loaded needed
    exports: {},
  });

  this.installedModules[moduleId](
    module,
    module.exports,
    this.__rspack_require__.bind(this),
    this.__rspack_dynamic_require__ && this.__rspack_dynamic_require__.bind(this)
  );

  return module.exports;
}

// mount require function
(function () {
  runtime.__rspack_require__ = __rspack_require__;
})();// The register function
function __rspack_register__(chunkIds, modules, callback) {
  if (
    chunkIds.some(
      function (id) {
        return this.installedChunks[id] !== 0;
      }.bind(this)
    )
  ) {
    for (moduleId in modules) {
      if (this.checkById(modules, moduleId)) {
        this.installedModules[moduleId] = modules[moduleId];
      }
    }
    if (callback) callback(this.__rspack_require__);
  }
  for (var i = 0; i < chunkIds.length; i++) {
    chunkId = chunkIds[i];
    if (this.checkById(this.installedChunks, chunkId) && this.installedChunks[chunkId]) {
      this.installedChunks[chunkId][0]();
    }
    this.installedChunks[chunkId] = 0;
  }
}

// mount register function
(function () {
  runtime.__rspack_register__ = __rspack_register__;
})(); })();