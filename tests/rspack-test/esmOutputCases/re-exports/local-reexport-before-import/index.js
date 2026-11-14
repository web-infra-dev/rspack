export { yv } from './y.js';
import './x.js';

it('keeps local side-effect order without externals', () => {
  expect(globalThis.__order).toEqual(['y', 'x']);
});
