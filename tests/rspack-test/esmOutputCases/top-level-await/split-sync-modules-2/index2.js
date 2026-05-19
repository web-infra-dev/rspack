import { value } from './lib.js';

const {value: valueAsync} = await import('./async.js');

it('should have correct value', () => {
  expect(value).toBe(valueAsync);
  expect(value()).toBe(42);
})