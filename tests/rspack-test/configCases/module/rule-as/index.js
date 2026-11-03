import explicit from './explicit.js';
import rematched from './rematch.js';
import priority from 'inline.rspack[json]!=!./priority.custom';

it('should apply Rule.as as an inline match resource', () => {
  expect(explicit).toBe('explicit source\n');
  expect(rematched).toBe('rematched source\n');
});

it('should prefer an inline match resource over Rule.as', () => {
  expect(priority).toEqual({ source: 'inline match resource' });
});
