const values = [
  require('./a'),
  require('./b'),
  require('./c'),
  require('./d'),
  require('./e'),
  require('./f'),
];
const stable = require('./stable');
const cleared = require('./cleared');
const trigger = require('./trigger');

it('should only rebuild modules which opt out of caching', async () => {
  expect(values).toEqual(['a', 'b', 'c', 'd', 'e', 'f']);
  expect(stable).toBe('stable');
  expect(cleared).toBe('cleared');
  expect(trigger).toBe(COMPILER_INDEX + 1);
  if (COMPILER_INDEX === 0) {
    await NEXT_START();
  }
});
