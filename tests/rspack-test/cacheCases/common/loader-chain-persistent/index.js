const value = require('./value');

it('should reuse a loader chain across compilers', async () => {
  expect(value).toBe(COMPILER_INDEX < 2 ? 1 : 2);
  if (COMPILER_INDEX === 0) {
    await NEXT_START();
  }
  if (COMPILER_INDEX === 1) {
    await NEXT_START();
  }
});
