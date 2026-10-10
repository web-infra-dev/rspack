const load = () => import(/* webpackChunkName: "loaded" */ './loaded');
const loadOther = () => import(/* webpackChunkName: "other" */ './other');

it('prepares new synchronous dependencies before self-accepted modules execute', async () => {
  expect((await load()).value).toBe('before');
  expect(__webpack_modules__['./dependency.js']).toBeUndefined();
  await NEXT_HMR();
  expect(__webpack_modules__['./dependency.js']).toBeTypeOf('function');
  expect((await load()).value).toBe('dependency');
  await NEXT_HMR();
  expect((await load()).value).toBe('updated dependency');
  expect((await loadOther()).value).toBe('updated dependency');
});
