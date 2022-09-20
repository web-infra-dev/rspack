function __rspack_dynamic_require__(chunkId) {
  return Promise.all(
    Object.keys(this)
      .filter(function (key) {
        return key.indexOf('rspack_load_dynamic') > 0;
      })
      .reduce(function (promises, key) {
        this[key](chunkId, promises);
        return promises;
      }.bind(this), [])
  );
}

// mount register dynamic require
(function () {
  runtime.__rspack_dynamic_require__ = __rspack_dynamic_require__;
})();