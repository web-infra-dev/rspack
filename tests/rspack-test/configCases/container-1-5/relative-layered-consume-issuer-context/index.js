it('should resolve a layered relative fallback from the issuer context', async () => {
  const value = await import('./nested/consumer');
  expect(value.default).toBe('issuer-relative-shared');
});
