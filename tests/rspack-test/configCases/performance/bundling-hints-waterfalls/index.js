it('loads shared chunks through deep and shallow paths with a cycle', async () => {
  const a = await import(/* webpackChunkName: 'a' */ './a');
  const b = await a.default();
  const deep = await b.default();
  const x = await import(/* webpackChunkName: 'x' */ './x');
  expect(await x.default()).toBe(deep);
  expect(await b.back()).toBe(a);
  expect(
    typeof deep.default === 'function' ? await deep.default() : deep.default,
  ).toBe('leaf');
});
