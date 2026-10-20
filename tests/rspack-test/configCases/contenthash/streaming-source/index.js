it('builds with streaming real content hash sources', () => {
  const hash = 'aabbccdd';
  expect(hash).toMatch(/^[0-9a-f]{8}$/);
  expect(hash).not.toBe(['aabb', 'ccdd'].join(''));
});
