it('should load an async chunk whose short id is reserved by a chunk name', async () => {
  const { default: value } = await import('./lazy');
  expect(value).toBe(42);
});
