import { value } from './json.js';

it('should separate JSON from an unused default export expression', () => {
  expect(value._version).toBe('1.2.3');
  expect(value.a).toBe(1);
});
