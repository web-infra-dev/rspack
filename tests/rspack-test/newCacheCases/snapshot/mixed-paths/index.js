const immutable = require('./consumer-immutable');
const sibling = require('./consumer-sibling');
const override = require('./consumer-override');
const managed = require('./consumer-managed');
const scoped = require('./consumer-scoped');
const nested = require('./consumer-nested');
const context = require('./consumer-context');
it('should apply webpack snapshot path classification', async () => {
  expect(immutable).toBe('1');
  for (const value of [sibling, override, context]) {
    expect(value).toBe(String(COMPILER_INDEX + 1));
  }
  // Changing nested package metadata does not replace the managed package root.
  for (const value of [managed, scoped, nested]) {
    expect(value).toBe(COMPILER_INDEX === 0 ? '1' : '2');
  }
  if (COMPILER_INDEX < 2) await NEXT_START();
});
