it('initializes federation for a remote in an async chunk', async () => {
  expect(__webpack_require__.federation.instance).toBeTruthy();
  expect((await import('remote/value')).default).toBe('remote value');
});
