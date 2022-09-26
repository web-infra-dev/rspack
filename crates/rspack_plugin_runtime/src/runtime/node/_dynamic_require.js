function __rspack_dynamic_require__(chunkId) {
  return new Promise(resolve => {
    this.install_chunk(require(this.__rspack_get_dynamic_chunk_url__(chunkId, 'js')));
    resolve();
  });
}

// mount register dynamic require
(function () {
  runtime.__rspack_dynamic_require__ = __rspack_dynamic_require__;
})();