it('intercepts factories and shares the live cache with refresh and test clients', () => {
  const id = require.resolve('./setup.cjs');
  const initial = require('./setup.cjs');
  const [cache, clientCache, factory, clientFactory] = (typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context).rstest_register_setup_id(id);
  expect(cache).toBe(clientCache);
  expect(cache.exports).toBe(initial);
  expect(factory).toBe(clientFactory);
  expect(typeof factory).toBe('function');
  expect((typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context).rstest_register_browser_test_entry(id)).not.toBe(initial);
  expect(__webpack_require__.$Refresh$.seen.filter(seen => seen === id)).toHaveLength(2);
  const missing = {};
  __webpack_require__.$Refresh$.intercept(missing);
  expect(missing.factory(null, initial)).toBe(initial);
});

it('uses the replaced namespace and default helpers and exposes a loader-defined asset base', () => {
  expect(__webpack_require__.pluginExports(42)).toEqual([true, 42, true, true]);
  expect(__webpack_require__.g).toBe(globalThis);
  expect(__webpack_require__.ab + 'addon.node').toBe('native-assets/addon.node');
});
