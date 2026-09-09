import esm from 'conditional';
const cjs = require('conditional');
const pick = require('./pick');
const loaded = require('./consumer');
it('should cache resolution by options and invalidate exports, loaders and missing candidates', async () => {
  expect(esm).toBe(COMPILER_INDEX < 2 ? 1 : 3);
  expect(cjs).toBe(COMPILER_INDEX < 2 ? 2 : 4);
  expect(pick).toBe(COMPILER_INDEX < 2 ? 1 : 2);
  expect(loaded).toBe(COMPILER_INDEX < 2 ? 1 : 2);
  if (COMPILER_INDEX < 2) await NEXT_START();
});
