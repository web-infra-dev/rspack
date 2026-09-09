it('loads a short async chain without a waterfall warning', async () => {
  const x = await import(/* webpackChunkName: 'x' */ './x');
  expect((await x.default()).default).toBe('leaf');
});
