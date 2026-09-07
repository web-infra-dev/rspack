var exports = {
  set value(value) {
    globalThis.__CJS_SHADOWED_WRITE_COUNT__ += value;
  },
};
exports.value = 1;

var module = {
  set exports(value) {
    globalThis.__CJS_SHADOWED_WRITE_COUNT__ += value;
  },
};
module.exports = 1;
