const result = require('./consumer');
it('should apply the configured snapshot strategy across compilers', async () => {
  expect(result.value).toBe('one');
  expect(result.runs).toBe(1);
  if (COMPILER_INDEX === 0) await NEXT_START();
});
