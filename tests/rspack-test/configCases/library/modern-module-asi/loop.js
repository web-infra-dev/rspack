import { calls } from './loop-dependency.js';

(calls.push('entry'));

it('should terminate the unbraced body of a loop before a trailing comment', () => {
  expect(calls).toEqual(['module', 'entry']);
});
