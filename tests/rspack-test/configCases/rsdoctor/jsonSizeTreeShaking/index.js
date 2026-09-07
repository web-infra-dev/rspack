import { used } from './data.json';

it('should use the tree-shaken JSON value', () => {
  expect(used).toBe('kept');
});
