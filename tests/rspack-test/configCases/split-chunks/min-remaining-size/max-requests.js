it('should check the single source left after applying request limits', async () => {
  const modules = await Promise.all([
    import(/* webpackChunkName: "one" */ './with-prelude'),
    import(/* webpackChunkName: "two" */ './async'),
  ]);
  expect(modules.map((module) => module.default)).toEqual(['shared:prelude', 'shared']);
});
