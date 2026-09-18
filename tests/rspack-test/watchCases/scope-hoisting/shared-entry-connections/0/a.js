import { foo } from './foo.js';

it('should preserve the first copied connection after rebuilding', () => {
  expect(foo()).toEqual({ value: WATCH_STEP === '0' ? 0 : 1 });
});
