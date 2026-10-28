import { bar } from './bar.js';

it('should use the shared entry after removing the second concatenation group', () => {
  expect(bar).toEqual({ value: 1 });
});
