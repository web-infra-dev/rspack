import changed from './changed';
import stable from './stable';

it('should restore valid modules and rebuild invalid modules', async () => {
  expect(stable).toBe('stable');
  expect((await import('./async')).default).toBe('async');
  const context = require.context('./context', false, /\.js$/);
  expect(context.keys()).toEqual(['./value.js']);
  expect(context('./value.js')).toBe('context');
  if (COMPILER_INDEX === 0) {
    expect(changed).toBe(1);
    await NEXT_START();
  } else {
    expect(changed).toBe(2);
  }
});
