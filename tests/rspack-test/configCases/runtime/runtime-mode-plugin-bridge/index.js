it('shares built-ins, private properties and the per-module loader with external runtime code', async () => {
  expect(__webpack_require__.probe().slice(0, 5)).toEqual(['/initial/', 17, 17, true, true]);
  expect(__webpack_public_path__).toBe('/changed/');
  expect(__webpack_require__.fromRequirements).toBe(1);
  if (EXPECT_GENERATE_OVERRIDE) expect(__webpack_require__.fromGenerate).toBe(2);
  expect(__webpack_require__.fromHook).toBe(42);
  expect(require('./subject.cjs')).toBe('./subject.cjs');
  expect(__webpack_require__.$Refresh$).toBeUndefined();
  expect((await import(/* webpackChunkName: "child" */ './child.js')).default).toBe('child');
  if (EXPECT_ASYNC_RUNTIME) {
    expect(__webpack_public_path__).toBe('/async/');
  }
});
