import value0 from './shared-0';
import value1 from './shared-1';
import value2 from './shared-2';
it('preserves shared values', () => {
  expect([value0, value1, value2]).toEqual([0, 1, 2]);
});
