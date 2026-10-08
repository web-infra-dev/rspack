import { value } from './shared.js';
it('should not need the asset in the unused runtime', () => {
  expect(value).toBe('unused runtime');
});
