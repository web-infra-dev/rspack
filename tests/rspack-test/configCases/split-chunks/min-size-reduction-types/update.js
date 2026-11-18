it('keeps consumers executable after another candidate extracts CSS', async () => {
  const [a, b] = await Promise.all([import('./a-update'), import('./b-update')]);
  expect(a.default).toBe('shared:a');
  expect(b.default).toBe('shared:b');
});
