it('retains federation for a shared consumer alongside local entries and Workers', async () => {
  expect((await import('shared-lib')).default).toBe('shared value');
  expect(__webpack_require__.federation.instance).toBeTruthy();
});
