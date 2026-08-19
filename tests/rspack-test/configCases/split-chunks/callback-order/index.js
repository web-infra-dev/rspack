it('preserves splitChunks callback order', async () => {
  const [{ a }, { b }, { c }] = await Promise.all([
    import(/* webpackChunkName: "a" */ './a'),
    import(/* webpackChunkName: "b" */ './b'),
    import(/* webpackChunkName: "c" */ './c'),
  ]);

  expect([a, b, c]).toEqual([2, 3, 4]);
  expect(globalThis.__sharedRuns).toBe(1);
});
