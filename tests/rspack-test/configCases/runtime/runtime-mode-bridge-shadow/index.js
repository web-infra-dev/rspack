it('emits extension code without a bridge when runtime names are only local', () => {
  expect(globalThis.runtimeShadowProbe).toEqual([7, '__webpack_require__']);
  expect(globalThis.lynx.__runtime_configs__.audit).toBe(42);
});
