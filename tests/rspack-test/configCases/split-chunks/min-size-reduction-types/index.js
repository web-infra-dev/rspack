it('keeps both consumers executable after pruning source types', async () => {
  const [a, b] = await Promise.all([import('./a'), import('./b')]);
  expect(a.default).toBe('shared:a');
  expect(b.default).toBe('shared:b');
});
