it('initializes federation for a consumer behind a local async boundary', async () => {
  expect(__webpack_require__.federation.instance).toBeTruthy();
  expect((await import('./consume-async.js')).default).toBe('shared value');
});
