it('emits extension code without a bridge when runtime names are only local', async () => {
  expect(globalThis.runtimeShadowProbe).toEqual([7, '__webpack_require__']);
  const lazy = await import('./lazy');
  expect(lazy.default).toBe(true);
});
