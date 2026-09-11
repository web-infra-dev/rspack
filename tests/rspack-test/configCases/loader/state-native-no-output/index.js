import value from './input.js';
it('does not mark an unexecuted native loader as finished on JS writeback', () => {
  expect(value).toBe('empty');
});
