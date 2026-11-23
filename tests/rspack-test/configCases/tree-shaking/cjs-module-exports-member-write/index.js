import './setup.js';
import './pure-member.cjs';
import './pure-computed-member.cjs';
import './reassigned-to-shared.cjs';
import './reassigned-to-global.cjs';
import './reassigned-in-function.cjs';
import './setter.cjs';
import './prototype-setter.cjs';
import './shadowed-module.cjs';
import './impure-rhs.cjs';
import './proto-member.cjs';

afterAll(() => {
  delete globalThis.__CJS_MEMBER_GLOBAL_WRITE__;
  delete globalThis.__CJS_MEMBER_SHARED_EXPORTS__;
  delete globalThis.__CJS_MEMBER_SETTER_COUNT__;
  delete globalThis.__CJS_MEMBER_PROTOTYPE_SETTER_COUNT__;
  delete globalThis.__CJS_MEMBER_SHADOWED_COUNT__;
  delete globalThis.__CJS_MEMBER_IMPURE_RHS_COUNT__;
  delete globalThis.__recordCjsMemberImpureRhs;
});

it('drops modules containing only module.exports member writes', () => {
  const source = require('fs').readFileSync(__filename, 'utf-8');
  const sentinels = [
    ['PURE', 'MODULE', 'EXPORTS', 'MEMBER', 'SENTINEL'],
    ['PURE', 'MODULE', 'EXPORTS', 'COMPUTED', 'MEMBER', 'SENTINEL'],
    ['PURE', 'MODULE', 'EXPORTS', 'FUNCTION', 'REASSIGN', 'SENTINEL'],
  ];

  for (const parts of sentinels) {
    expect(source).not.toContain(parts.join('_'));
  }
});

it('keeps module.exports member writes after module.exports is reassigned to another object', () => {
  expect(globalThis.__CJS_MEMBER_SHARED_EXPORTS__.fromReassigned).toBe(true);
  expect(globalThis.__CJS_MEMBER_GLOBAL_WRITE__).toBe(1);
});

it('keeps module.exports member writes that can reach a setter', () => {
  expect(globalThis.__CJS_MEMBER_SETTER_COUNT__).toBe(1);
  expect(globalThis.__CJS_MEMBER_PROTOTYPE_SETTER_COUNT__).toBe(1);
});

it('keeps member writes through a shadowed module binding', () => {
  expect(globalThis.__CJS_MEMBER_SHADOWED_COUNT__).toBe(1);
});

it('keeps module.exports member writes with an impure right-hand side', () => {
  expect(globalThis.__CJS_MEMBER_IMPURE_RHS_COUNT__).toBe(1);
});

it('keeps module.exports.__proto__ writes', () => {
  const source = require('fs').readFileSync(__filename, 'utf-8');
  expect(source).toContain(
    ['PROTO', 'MODULE', 'EXPORTS', 'MEMBER', 'SENTINEL'].join('_'),
  );
});
