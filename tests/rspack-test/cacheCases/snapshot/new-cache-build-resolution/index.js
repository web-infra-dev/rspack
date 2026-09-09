const builds = require('./consumer');
it('should invalidate only when build dependencies resolve to a different target', async () => {
  expect(builds).toBe(COMPILER_INDEX < 2 ? 1 : 2);
  if (COMPILER_INDEX < 2) await NEXT_START();
});
