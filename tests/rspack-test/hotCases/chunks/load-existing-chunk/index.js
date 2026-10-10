module.hot.accept();

it('updates a loaded chunk listed in both c and load only once', async () => {
  await import(/* webpackChunkName: "other" */ './other');
  expect(globalThis.loadOtherValue).toBe('before');
  const hash = __webpack_hash__;
  const check = module.hot.check;
  let updated;
  module.hot.check = (...args) => check(...args).then(ids => {
    updated = ids;
    return ids;
  });
  try {
    await NEXT_HMR();
  } finally {
    module.hot.check = check;
  }
  const manifest = readUpdateManifest(hash);
  expect(manifest.load.main).toContain('shared');
  expect(manifest.c).toContain('shared');
  expect(updated.filter(id => id === './shared.js')).toHaveLength(1);
  expect(globalThis.loadMainValue).toBe('after');
  expect(globalThis.loadOtherValue).toBe('after');
});
---
import { value } from './shared';
globalThis.loadMainValue = value;
globalThis.loadOther = () => import(/* webpackChunkName: "other" */ './other');
module.hot.accept();
