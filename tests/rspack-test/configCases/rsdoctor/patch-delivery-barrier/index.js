import { value } from './lib';

it('should deliver every rsdoctor patch before the compilation finishes', () => {
  expect(value).toBe(42);
});
