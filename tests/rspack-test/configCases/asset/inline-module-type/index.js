import normal from './target.txt';
import single from '!./target.txt';
import pre from '-!./target.txt';
import double from '!!./target.txt';
import matched from './virtual.txt!=!!!./target.txt';
import explicit from './virtual.webpack[asset/source]!=!!!./target.txt';

it('should ignore Rule.type only for the !! prefix', () => {
  const source = 'module.exports = require("value");\n';
  expect(normal).toBe(source);
  expect(single).toBe(source);
  expect(pre).toBe(source);
  expect(double).toBe('javascript');
  expect(matched).toBe('javascript');
  expect(explicit).toBe(source);
});
