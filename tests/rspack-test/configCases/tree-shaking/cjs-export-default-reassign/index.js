import def from './dep.cjs';
import { foo } from './named.cjs';
import './setup-nested-free-read.js';
import './nested-free.cjs';

it('keeps the reassigned CJS default export value (innerGraph on, production)', () => {
  // #14589: the chained `var _default = (exports.default = value)` write must
  // NOT be folded into the unused `_default` local and dropped.
  expect(def).toBe(42);
});

it('keeps a reassigned CJS named export value', () => {
  expect(foo).toBe(7);
});

it('keeps nested free identifier reads in CJS export RHS observable', () => {
  expect(globalThis.__CJS_NESTED_FREE_READ_COUNT__).toBe(1);
});
