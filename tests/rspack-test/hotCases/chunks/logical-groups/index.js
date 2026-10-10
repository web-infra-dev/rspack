import { load, loadUnloaded, loadContext } from './loader';
import './trigger';

// No accept boundary: changing only the loading plan must not reach this entry.
it('updates loaded groups without invalidating their importers or module instances', async () => {
  const first = await load();
  const firstContext = await loadContext("loaded");
  const factory = __webpack_modules__[require.resolve('./loader')];
  expect(__webpack_modules__["./shared-unloaded.js"]).toBeUndefined();
  for (let i = 0; i < 2; i++) {
    await NEXT_HMR();
    expect(__webpack_modules__[require.resolve('./loader')]).toBe(factory);
    expect(await load()).toBe(first);
    expect(await loadContext("loaded")).toBe(firstContext);
    expect(__webpack_modules__["./shared-unloaded.js"]).toBeUndefined();
  }
  // Import a previously unrequested group after check(), before apply().
  const check = module.hot.check;
  module.hot.check = () => check(false);
  try { await NEXT_HMR(); } finally { module.hot.check = check; }
  expect(module.hot.status()).toBe('ready');
  const late = await loadUnloaded();
  await module.hot.apply();
  expect(await loadUnloaded()).toBe(late);
  expect((await loadContext('unloaded')).value).toBe('unloaded');
});
