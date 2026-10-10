import { instance, invalidate } from './shared';

it('honors explicit invalidation during an initial chunk migration', async () => {
  let accepted = 0;
  module.hot.accept('./shared', () => accepted++);
  for (const phase of ['prepare', 'check', 'ready']) {
    const hash = __webpack_hash__;
    const before = instance;
    const previousAccepted = accepted;
    if (phase === 'ready') {
      const check = module.hot.check;
      module.hot.check = () => check(false);
      try {
        await NEXT_HMR();
      } finally {
        module.hot.check = check;
      }
      expect(module.hot.status()).toBe('ready');
      invalidate();
      await module.hot.apply();
    } else {
      const onStatus = status => {
        if (status === phase) invalidate();
      };
      module.hot.addStatusHandler(onStatus);
      await NEXT_HMR();
      module.hot.removeStatusHandler(onStatus);
    }
    expect({ accepted, sameInstance: instance === before }).toEqual({
      accepted: previousAccepted + 1,
      sameInstance: false,
    });
    const manifest = readUpdateManifest(hash);
    expect(manifest.load.main).toContain('shared');
    expect(manifest.c).not.toContain('shared');
    expect(manifest).not.toHaveProperty('initial');
    if (phase !== 'ready') {
      // Merge back so the next phase can exercise a new initial dependency.
      await NEXT_HMR();
    }
  }
});
