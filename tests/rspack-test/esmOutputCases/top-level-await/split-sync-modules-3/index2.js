import { value } from './lib.js';

const {value: valueAsync1} = await import('./async1.js');


it('should have correct value', async () => {
  expect(value).toBe(valueAsync1);
  const {value: valueAsync2} = await import('./async2.js');
  expect(valueAsync1).toBe(valueAsync2);
  expect(value()).toBe(42);
})