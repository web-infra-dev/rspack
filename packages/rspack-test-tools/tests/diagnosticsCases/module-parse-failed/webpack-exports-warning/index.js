(async function () {
  const { a, b } = await import(/* webpackExports: ["a", "b", "c"] */ './lib');
  (a, b)
})();