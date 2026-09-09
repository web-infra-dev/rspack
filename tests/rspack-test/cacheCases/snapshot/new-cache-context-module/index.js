const sync = require.context('./context-fixture', false, /\.js$/, 'sync');
const lazy = require.context('./context-fixture', false, /\.js$/, 'lazy');
it('should restore context dependencies and async blocks and invalidate changes', async () => {
  const keys = COMPILER_INDEX === 3 ? ['./a.js', './b.js'] : ['./a.js'];
  expect(sync.keys()).toEqual(keys);
  expect(lazy.keys()).toEqual(keys);
  expect(sync('./a.js')).toBe(COMPILER_INDEX < 2 ? 1 : 2);
  expect(await lazy('./a.js')).toBe(COMPILER_INDEX < 2 ? 1 : 2);
  if (COMPILER_INDEX === 3) {
    expect(sync('./b.js')).toBe(3);
    expect(await lazy('./b.js')).toBe(3);
  }
  if (COMPILER_INDEX < 3) await NEXT_START();
});
