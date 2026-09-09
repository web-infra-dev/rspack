const builds = require('./consumer');
it('should track missing higher-priority build dependency candidates', async () => {
  expect(builds).toBe(COMPILER_INDEX + 1);
  if (COMPILER_INDEX === 0) await NEXT_START();
});
