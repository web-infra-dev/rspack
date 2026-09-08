import engines from './engines.cjs';
import reassigned from './reassign.cjs';
import declarations from './declarations.cjs';

it('should preserve CommonJS exports assignments', () => {
  expect(engines).toEqual({ yaml: 1 });
  expect(reassigned).toEqual({ original: {}, reassigned: { value: 3 } });
});

it('should deconflict destructured exports declarations in automatic modules', () => {
  expect(declarations).toEqual({ object: 42, array: 43 });
});

it('should preserve property keys in defaulted shorthand bindings', () => {
  const defaulted = require('./declaration-default.js');
  expect(defaulted.exports).toBe(42);
  expect(defaulted.read({ exports: 43 })).toBe(43);
  expect(defaulted.read({})).toBe(7);
});

it('should reject undeclared exports assignments in strict ES modules', () => {
  expect(() => require('./assignment-esm.mjs')).toThrow(ReferenceError);
});

it('should preserve assignments to declared exports in strict ES modules', () => {
  expect(require('./declaration-esm.mjs').exports).toBe(43);
});

export default engines;
