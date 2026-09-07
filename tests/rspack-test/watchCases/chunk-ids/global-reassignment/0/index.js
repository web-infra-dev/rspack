it('should invalidate cached code generation when global chunk ids change', async () => {
  const targetChunk = __STATS__.chunks.find((chunk) =>
    chunk.modules?.some((module) => module.name === './chunk77.js'),
  );
  expect(targetChunk).toBeTruthy();

  if (WATCH_STEP === '0') {
    STATE.targetChunkId = targetChunk.id;
  } else {
    expect(targetChunk.id).not.toBe(STATE.targetChunkId);
  }

  const value = await import('./chunk77.js');
  expect(value.default).toBe('target');
});
