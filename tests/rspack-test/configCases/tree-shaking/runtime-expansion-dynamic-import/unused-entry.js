it('should initially use only the live export from the shared chunk', async () => {
  const { live } = await import(/* webpackChunkName: "shared" */ './shared');
  expect(live).toBe('live');
});
