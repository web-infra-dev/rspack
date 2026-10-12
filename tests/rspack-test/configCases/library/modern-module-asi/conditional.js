import { calls } from './conditional-dependency.js';

(calls.push('entry'));

it('should terminate the unbraced body of a conditional', () => {
  expect(calls).toEqual(['entry']);
});
