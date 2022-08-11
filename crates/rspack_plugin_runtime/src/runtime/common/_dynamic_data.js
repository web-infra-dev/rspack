(function () {
  runtime.installedCssChunks = {};
})();

(function () {
  runtime.chunkHashData = {
    js: {},
    css: {}
  };
})();

(function () {
  runtime.setChunkHashData = function (chunkId, hash, type) {
    return this.chunkHashData[type][chunkId] = hash;
  };
})();

(function () {
  runtime.__rspack_has_dynamic_chunk__ = function (chunkId, type) {
    return Boolean(this.chunkHashData && this.chunkHashData[type] && this.chunkHashData[type][chunkId]);
  };
})();

