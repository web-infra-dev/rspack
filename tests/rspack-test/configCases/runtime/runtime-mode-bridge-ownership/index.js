it('installs later helpers on the replacement loader at the provider stage', async () => {
  const loader = __webpack_require__;
  expect(loader.privateState.before).not.toContain('p');
  expect(loader.privateState.before).not.toContain('u');
  expect(loader.p).toBe('/initial/');
  expect(loader.u('lazy')).toMatch(/lazy/);
  expect(__webpack_require__.privateState).toBe(loader.privateState);
  expect((await import(/* webpackChunkName: "lazy" */ './lazy')).default).toBe(42);
});
