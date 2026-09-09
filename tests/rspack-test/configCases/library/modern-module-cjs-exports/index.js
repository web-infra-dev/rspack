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

it('should deconflict destructured runtime exports bindings', () => {
  expect({ ...require('./declaration-runtime-object.js') }).toEqual({
    rspackExports: 42,
    __rspack_exports: 43,
    __webpack_exports__: 44,
  });
  const array = require('./declaration-runtime-array.js');
  expect({ ...array }).toEqual({
    rspackExports: 43,
    __webpack_exports__: 45,
    update: expect.any(Function),
  });
  array.update();
  expect(array.rspackExports).toBe(44);
  expect(array.__webpack_exports__).toBe(46);

  const defaulted = require('./declaration-runtime-default.js');
  expect(defaulted.rspackExports).toBe(42);
  expect(defaulted.__webpack_exports__).toBe(44);
  expect(defaulted.read({ rspackExports: 43, __webpack_exports__: 45 })).toEqual([
    43,
    45,
  ]);
  expect(defaulted.read({})).toEqual([7, 9]);
});

it('should preserve global exports assignments in strict ES modules', () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'exports');
  const original = { original: true };
  globalThis.exports = original;
  try {
    const namespace = require('./assignment-global-esm.mjs');
    expect(namespace.before).toBe(original);
    expect(namespace.after).toEqual({ value: 1 });
    expect(namespace.after).toBe(globalThis.exports);

    const reassigned = { value: 2 };
    expect(namespace.assign(reassigned)).toBe(reassigned);
    expect(globalThis.exports).toBe(reassigned);

    const destructured = { value: 3 };
    expect(namespace.destructure({ exports: destructured })).toBe(destructured);
    expect(globalThis.exports).toBe(destructured);

    delete globalThis.exports;
    expect(() => namespace.assign({})).toThrow(ReferenceError);
    expect(() => namespace.destructure({ exports: {} })).toThrow(ReferenceError);
    expect(Object.hasOwn(globalThis, 'exports')).toBe(false);
  } finally {
    if (descriptor) {
      Object.defineProperty(globalThis, 'exports', descriptor);
    } else {
      delete globalThis.exports;
    }
  }
});

export default engines;
