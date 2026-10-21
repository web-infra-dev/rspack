it('should split a module shared by multiple source chunks', async () => {
  const modules = await Promise.all([
    import(/* webpackChunkName: "one" */ './async?one'),
    import(/* webpackChunkName: "two" */ './async?two'),
  ]);
  expect(modules.map((module) => module.default)).toEqual(['shared', 'shared']);
});
