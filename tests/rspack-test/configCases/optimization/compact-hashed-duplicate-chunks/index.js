it('should assign unique ids to async chunks with identical modules', async () => {
  const { default: value } = await import('./shared');
  expect(value).toBe(42);
});
