import { local } from './named.js';
import localDefault from './default.js';
import forwardedDefault from './forwarded.js';

it('should preserve local and forwarded export values', () => {
  expect(local()).toBe('named');
  expect(localDefault()).toBe('default');
  expect(forwardedDefault()).toBe('default');
});
