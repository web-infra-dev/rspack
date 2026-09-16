import { value } from './shared';
import { value as duplicate } from './shared';

it('preserves module memberships across inline storage and hash storage', () => {
  expect(value).toBe(42);
  expect(duplicate).toBe(value);
});
