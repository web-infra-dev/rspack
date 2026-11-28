var module = {
  exports: {
    set value(value) {
      globalThis.__CJS_MEMBER_SHADOWED_COUNT__ =
        (globalThis.__CJS_MEMBER_SHADOWED_COUNT__ ?? 0) + value;
    },
  },
};
module.exports.value = 1;
