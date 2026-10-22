import './input.js?empty';
import './input.js?newlines';
import './input.js?crlf';
import './input.js?whitespace';
import './input.js?unicode';
import { value } from './input.js?mapped';

it('preserves modules with absent or empty source maps', () => {
  expect(value).toBe('mapped');
});
