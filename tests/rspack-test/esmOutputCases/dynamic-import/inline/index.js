it('should have correct inline export', async () => {
  const { value } = await import('./foo')
  expect(value).toBe(42)
})
