it('should execute an entry with its runtime and load an async chunk', async () => {
  expect(['main', 'secondary', 'third', 'runtime']).toContain(__webpack_runtime_id__);
  const { value } = await import('./async');
  expect(value).toBe(42);
});
