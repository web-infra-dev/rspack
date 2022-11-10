(function () {
  runtime.__rspack_get_dynamic_chunk_url__ = function (chunkId, type) {
    return chunkId + __GET_DYNAMIC_URL_HASH_PLACEHOLDER__ + "." + type;
  };
})();
