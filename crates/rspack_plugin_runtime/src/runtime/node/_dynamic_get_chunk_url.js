(function () {
  runtime.__rspack_get_dynamic_chunk_url__ = function (chunkId, type) {
    return './static/' + type + '/' + chunkId + __GET_DYNAMIC_URL_HASH_PLACEHOLDER__ + '.chunk.' + type;
  };
})();
