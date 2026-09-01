const a = require('./a');
const b = require('./b');
const stable = require('./stable');
const trigger = require('./trigger');

it('should respect cacheability changes from succeedModule', async () => {
  expect(a).toBe('a');
  expect(b).toBe('b');
  expect(stable).toBe('stable');
  expect(trigger).toBe(COMPILER_INDEX + 1);
  if (COMPILER_INDEX === 0) {
    await NEXT_START();
  }
});
