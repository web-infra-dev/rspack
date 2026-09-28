import { foo } from './foo.js';

it('should recreate the second copied connection', () => {
  expect(foo()).toEqual({ value: 1 });
});
