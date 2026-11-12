import { helpers } from '@module-federation/runtime-tools/runtime';

it('resolves public runtime helper subpaths during instance initialization', async () => {
  expect(typeof helpers.utils.getRemoteInfo).toBe('function');
  const container = require('./container.js');
  await container.init({});
  const factory = await container.get('./module');
  expect(factory().helpersReady).toBe(true);
  delete globalThis.__mfRuntimeHelpersReady;
});
