function __rspack_dynamic_require__(chunkId) {
  return new Promise(resolve => resolve(require(this.__rspack_get_dynamic_chunk_url__(chunkId, 'js'))));
}

// mount register dynamic require
(function () {
  runtime.__rspack_dynamic_require__ = __rspack_dynamic_require__;
})();