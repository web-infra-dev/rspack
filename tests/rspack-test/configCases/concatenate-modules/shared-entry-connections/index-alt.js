import { foo } from './foo.js';

it('should retain the shared entry module in the second runtime', () => {
  expect(foo()).toEqual({ value: 'bar' });
});
