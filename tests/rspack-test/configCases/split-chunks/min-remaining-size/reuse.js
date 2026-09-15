it('should check the source remaining after excluding the reused destination', async () => {
  const modules = await Promise.all([
    import(/* webpackChunkName: "shared" */ './shared'),
    import(/* webpackChunkName: "async" */ './async'),
  ]);
  expect(modules.map((module) => module.default)).toEqual(['shared', 'shared']);
});
