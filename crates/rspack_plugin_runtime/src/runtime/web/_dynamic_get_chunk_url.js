(function () {
  runtime.__rspack_get_dynamic_chunk_url__ = function (chunkId, type) {
    return 'static/' + type + '/' + chunkId + '.' + this.chunkHashData[type][chunkId] + '.chunk.' + type;
  };
})();