import { load } from './calls';
it('recomputes array loading when the chunk list changes', async () => {
  expect((await load()).default).toBe(WATCH_STEP === '1' || WATCH_STEP === 1 ? 'single' : 'shared');
  expect(__webpack_chunk_load__.toString().includes('Array.isArray')).toBe(!(WATCH_STEP === '1' || WATCH_STEP === 1));
});
