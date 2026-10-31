it('should update collision references without regenerating the unrelated leaf', async () => {
  const chunk = __STATS__.chunks.find(chunk => chunk.modules?.some(module => module.name === './chunk77.js'));
  if (WATCH_STEP === '0') {
    expect(chunk.id).toBe(927);
    STATE.targetId = chunk.id;
    STATE.stableId = __STATS__.chunks.find(chunk => chunk.modules?.some(module => module.name === './stable.js')).id;
  }
  else {
    expect(chunk.id).not.toBe(STATE.targetId);
    expect(__STATS__.chunks.find(chunk => chunk.modules?.some(module => module.name === './stable.js')).id).toBe(STATE.stableId);
  }
  expect((await import('./chunk77.js')).default).toBe('target');
  expect((await import('./stable.js')).default).toBe('stable');
});
