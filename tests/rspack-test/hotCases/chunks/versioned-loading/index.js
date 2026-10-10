const load = () => import(/* webpackChunkName: "loaded" */ './loaded');

it('loads the installed build while newer compilations wait for HMR', async () => {
  const check = module.hot.check;
  module.hot.check = () => Promise.resolve([]);
  try {
    await NEXT_HMR();
  } finally {
    module.hot.check = check;
  }
  // The server is ahead. The old group plan still needs the old factory.
  expect((await load()).value).toBe('before');
  expect(__webpack_modules__['./after.js']).toBeUndefined();
  await module.hot.check(true);
  expect((await load()).value).toBe('after');
  await NEXT_HMR();
  expect((await load()).value).toBe('latest');
});
