import './pkg/unused.js';
import { value } from './pkg/index.js';

it('matches individual description fields and preserves sideEffects', () => {
  expect(value).toBe('matched');
});
