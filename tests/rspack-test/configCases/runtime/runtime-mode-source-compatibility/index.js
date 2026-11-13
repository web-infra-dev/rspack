it('preserves the source hook behavior of each runtime mode', async () => {
  expect(__webpack_require__.compatEmpty).toBe(RSPACK_MODE ? undefined : 'original');
  expect(__webpack_require__.compatGenerate).toBe(RSPACK_MODE ? 'overridden' : 'original');
  expect(__webpack_require__.compatStages).toBe(
    RSPACK_MODE ? 'original-first-second' : 'original-second',
  );
  expect(__webpack_require__.compatMapped).toBe('modified');

  const map = JSON.parse(
    require('fs').readFileSync(require('path').join(__dirname, RUNTIME_SOURCE_MAP), 'utf8'),
  );
  expect(map.sources.some(source => source.includes('runtime-compat-mapped'))).toBe(RSPACK_MODE);

  globalThis.runtimeCompatAsync = false;
  expect((await import(/* webpackChunkName: "compat-async" */ './async.js')).value).toBe(42);
  expect(globalThis.runtimeCompatAsync).toBe(!ESM_OUTPUT || RSPACK_MODE);
  delete globalThis.runtimeCompatAsync;
});
