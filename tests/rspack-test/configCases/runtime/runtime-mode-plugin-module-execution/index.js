it('intercepts factories and shares the live cache with refresh and test clients', () => {
  const id = require.resolve('./setup.cjs');
  const initial = require('./setup.cjs');
  const [cache, clientCache, factory, clientFactory] = __webpack_require__.rstestCacheEntry(id);
  expect(cache).toBe(clientCache);
  expect(cache.exports).toBe(initial);
  expect(initial.refreshId).toBe(id);
  expect(factory).toBe(clientFactory);
  expect(typeof factory).toBe('function');
  expect(__webpack_require__.rstestIds).toEqual({ setups: [id], tests: { test: id }, browser: { browser: id } });
  expect(__webpack_require__.rstestResetModule(id)).not.toBe(initial);
  expect(__webpack_require__.rstestIds.setups).toEqual([id, id]);
  expect(__webpack_require__.rstestProxyCalls.filter(seen => seen === id)).toHaveLength(2);
  expect(__webpack_require__.$Refresh$.seen.filter(seen => seen === id)).toHaveLength(2);
  expect(__webpack_require__.$Refresh$.loadFromBackground(id).refreshId).toBe(`${id}(react:background)`);
  expect(globalThis.$RefreshReg$).toBeUndefined();
  const missing = { id: './missing.cjs' };
  __webpack_require__.$Refresh$.intercept(missing);
  expect(missing.factory(null, initial)).toBe(initial);
});

it('uses the replaced namespace and default helpers and exposes a loader-defined asset base', () => {
  expect(__webpack_require__.pluginExports(42)).toEqual([true, 42, true, true]);
  expect(__webpack_require__.g).toBe(globalThis);
  expect(require('./asset.txt')).toBe('relocated');
  expect(globalThis.aureliaWebpackRequire.m).toBe(__webpack_require__.m);
  expect(globalThis.aureliaWebpackRequire.c).toBe(__webpack_require__.c);
});
