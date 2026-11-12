import x from './x';
import y from './y';

it('should load x and y', () => {
  expect([x, y]).toEqual(['x', 'y']);
});
