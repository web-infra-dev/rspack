it('should initialize a fallback remote with each configured share scope', async () => {
  const shareScopeKeys = await import('remote/value');

  expect(shareScopeKeys.default).toEqual(['scope1', 'scope2']);
});
