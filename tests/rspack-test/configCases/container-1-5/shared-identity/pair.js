const federation = __webpack_require__.federation;
federation.load = () =>
  Promise.all([import('one'), import('two')]).then((values) =>
    values.map((value) => value.default),
  );
module.exports = federation;
