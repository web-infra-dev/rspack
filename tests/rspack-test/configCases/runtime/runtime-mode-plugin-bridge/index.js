it('shares built-ins, private properties and the per-module loader with external runtime code', async () => {
  expect(__webpack_require__.probe().slice(0, 5)).toEqual(['/initial/', 17, 17, true, true]);
  expect(__webpack_public_path__).toBe('/changed/');
  if (EXPECT_EARLY_RUNTIME) expect(__webpack_require__.earlyResult).toBe(require('./early.cjs'));
  __webpack_require__.pluginState = 23;
  expect(__webpack_require__.readPluginState()).toBe(23);
  expect(__webpack_require__.fromRequirements).toBe(1);
  if (EXPECT_GENERATE_OVERRIDE) expect(__webpack_require__.fromGenerate).toBe(2);
  expect(__webpack_require__.fromHook).toBe(42);
  const [refresh, readLater, value] = require('./subject.cjs');
  expect(refresh).toBe('./subject.cjs');
  expect(readLater()).toBe(42);
  expect(value).toBe(42);
  expect(__webpack_require__.$Refresh$).toBeUndefined();
  expect((await import(/* webpackChunkName: "child" */ './child.js')).default).toBe('child');
  if (EXPECT_ASYNC_RUNTIME) {
    expect(__webpack_public_path__).toBe('/async/');
  }
  __webpack_public_path__ = '/from-app/';
  expect(__webpack_require__.readPublicPath()).toEqual(['/from-app/', '/from-app/']);
});
