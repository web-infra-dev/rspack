it('should load the added collision chunk', async () => {
  expect((await import('./chunk20.js')).default).toBe('collision');
});
