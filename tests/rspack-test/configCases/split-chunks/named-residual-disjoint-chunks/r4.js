import x from './x';
import z from './z';

it('should load x and z', () => {
  expect([x, z]).toEqual(['x', 'z']);
});
