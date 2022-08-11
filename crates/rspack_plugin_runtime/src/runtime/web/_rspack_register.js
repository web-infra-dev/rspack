// The register function
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
})();