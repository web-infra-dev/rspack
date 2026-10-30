import { load, loadAgain } from './loader';

it('installs new dependencies before reexecuting a named group after its imports move', async () => {
  expect((await load()).value).toBe(1);
  expect((await loadAgain()).value).toBe(1);
  await NEXT_HMR();
  // Check self-accept before another import can load the new shared chunk.
  expect(globalThis.namedGroupValue).toBe(2);
  expect(module.hot.status()).toBe('idle');
  expect((await require('./loader').load()).value).toBe(2);
});
