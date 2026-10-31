it('should retain chunk ids and load both leaves after a content-only edit', async () => {
  const changed = await import('./changed.js');
  const stable = await import('./stable.js');
  expect(changed.default).toBe(WATCH_STEP);
  expect(stable.default).toBe('stable');
  const ids = __STATS__.chunks.map(chunk => chunk.id).sort();
  if (WATCH_STEP === '0') STATE.chunkIds = ids;
  else expect(ids).toEqual(STATE.chunkIds);
});
