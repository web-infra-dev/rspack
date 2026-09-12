it('loads chunk arrays with importScripts', async () => {
  const load = () => import('./lazy');
  const [a, b] = await Promise.all([load(), load()]);
  expect(a.default).toBe('ok');
  expect(a).toBe(b);
  expect((await load()).default).toBe('ok');
});
