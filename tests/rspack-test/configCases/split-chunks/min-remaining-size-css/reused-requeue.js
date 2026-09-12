it('should keep the reused destination excluded when retrying a smaller candidate', async () => {
  const [, module] = await Promise.all([
    import(/* webpackChunkName: "reuse" */ './controls.css'),
    import(/* webpackChunkName: "controls" */ './css-wrapper'),
  ]);
  expect(module.default).toBe('wrapped');
});
