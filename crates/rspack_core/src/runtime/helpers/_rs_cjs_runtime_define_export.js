rs._define_export = (exports, definition) => {
  for (var key in definition) {
    if (rs._o(definition, key) && !rs._o(exports, key)) {
      Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
    }
  }
};
