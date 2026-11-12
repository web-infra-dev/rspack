import { hasAnimation } from './motion/es/util/motion.js';
import { CSSMotion } from './motion/es/CSSMotion.js';

it('preserves shorthand assignment keys after module concatenation and minification', () => {
  expect(hasAnimation({ style: { animation: 'none' } })).toBe(true);
  expect(CSSMotion({ style: 'none' })).toBe('none');
});
