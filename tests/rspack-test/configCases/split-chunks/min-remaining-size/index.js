it('should load the asynchronous module after checking the remaining size', async () => {
  const module = await import(/* webpackChunkName: "async" */ './async');
  expect(module.default).toBe('shared');
});
