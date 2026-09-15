import def from './dep.cjs';
import { foo } from './named.cjs';
import './setup.js';
import './pure-module-exports.cjs';
import './pure-named-exports.cjs';
import './impure-named-exports.cjs';
import './dynamic-key.cjs';
import './shadowed-cjs-globals.cjs';
import './esm-exports-assignment.js';
import './nested-export-write.cjs';
import './reassigned-module-exports-write.cjs';
import './proto-export-write.cjs';
import './computed-proto-export-write.cjs';
import './pure-chained-exports.cjs';

afterAll(() => {
  delete globalThis.__CJS_DYNAMIC_KEY_READ_COUNT__;
  delete globalThis.__CJS_IMPURE_RHS_COUNT__;
  delete globalThis.__CJS_SHADOWED_WRITE_COUNT__;
  delete globalThis.__getCjsDynamicExportKey;
  delete globalThis.__recordCjsImpureRhs;
});

it('keeps the reassigned CJS default export value (innerGraph on, production)', () => {
  // #14589: the chained `var _default = (exports.default = value)` write must
  // not be folded into the unused `_default` local and dropped.
  expect(def).toBe(42);
});

it('keeps a reassigned CJS named export value', () => {
  expect(foo).toBe(7);
});

it('drops modules containing only pure CJS export assignments', () => {
  const source = require('fs').readFileSync(__filename, 'utf-8');
  const sentinels = [
    ['PURE', 'MODULE', 'EXPORTS', 'SENTINEL'],
    ['PURE', 'COMPUTED', 'MODULE', 'EXPORTS', 'SENTINEL'],
    ['PURE', 'NAMED', 'EXPORTS', 'SENTINEL'],
    ['PURE', 'CHAINED', 'EXPORTS', 'SENTINEL'],
  ];

  for (const parts of sentinels) {
    expect(source).not.toContain(parts.join('_'));
  }
});

it('keeps side effects detected by the existing parser', () => {
  expect(globalThis.__CJS_IMPURE_RHS_COUNT__).toBe(1);
  expect(globalThis.__CJS_DYNAMIC_KEY_READ_COUNT__).toBe(1);
  expect(globalThis.__CJS_SHADOWED_WRITE_COUNT__).toBe(2);
});

it('does not relax other assignment targets', () => {
  const source = require('fs').readFileSync(__filename, 'utf-8');
  const sentinels = [
    ['ESM', 'EXPORTS', 'ASSIGNMENT', 'SENTINEL'],
    ['NESTED', 'EXPORT', 'WRITE', 'SENTINEL'],
    ['REASSIGNED', 'MODULE', 'EXPORTS', 'SENTINEL'],
    ['PROTO', 'EXPORT', 'WRITE', 'SENTINEL'],
    ['COMPUTED', 'PROTO', 'EXPORT', 'WRITE', 'SENTINEL'],
  ];

  for (const parts of sentinels) {
    expect(source).toContain(parts.join('_'));
  }
});
