import esmDefault, * as esmNamespace from 'esm-ext';
import cjsDefault, * as cjsNamespace from 'cjs-ext';
import plainDefault from 'plain-ext';
import undefinedDefault from 'undefined-ext';
import unflaggedDefault, { namedExport as unflaggedNamed } from 'unflagged-ext';
import selectedDefault, { namedExport as selectedNamed } from 'selected-ext';
import callbackDefault from 'callback-ext';
import syncDefault from 'sync-ext';
import promiseDefault from 'promise-ext';
import * as reexports from './reexport';

it('should use the namespace default with esModule interop', () => {
  expect(esmDefault).toBe('the default');
  expect(esmNamespace.default).toBe('the default');
  expect(esmNamespace.namedExport).toBe(42);
  expect(unflaggedDefault).toBe('unflagged default');
  expect(unflaggedNamed).toBe(44);
});

it('should preserve the whole exports with default interop', () => {
  expect(cjsDefault.__esModule).toBe(true);
  expect(cjsDefault.default).toBe('the default');
  expect(cjsNamespace.default).toBe(cjsDefault);
  expect(cjsNamespace.namedExport).toBe(42);
});

it('should preserve importer-driven interop without a hint', () => {
  expect(plainDefault).toBe(STRICT_IMPORTER ? cjsDefault : esmDefault);
  expect(undefinedDefault).toBe(plainDefault);
});

it('should preserve interop through reexports', () => {
  expect(reexports.esmDefault).toBe(esmDefault);
  expect(reexports.esmNamed).toBe(42);
  expect(reexports.cjsDefault).toBe(cjsDefault);
  expect(reexports.cjsNamed).toBe(42);
});

it('should support callback, synchronous, and promise externals', () => {
  expect(callbackDefault).toBe(esmDefault);
  expect(syncDefault).toBe(cjsDefault);
  expect(promiseDefault).toBe(esmDefault);
});

it('should ignore the hint for property-selected externals', () => {
  expect(selectedDefault).toBe(STRICT_IMPORTER ? cjsDefault.nested : 'nested default');
  expect(selectedNamed).toBe(43);
});

it('should apply interop to dynamic imports', async () => {
  const [esm, cjs, plain] = await Promise.all([
    import('esm-ext'),
    import('cjs-ext'),
    import('plain-ext'),
  ]);
  expect(esm.default).toBe(esmDefault);
  expect(esm.namedExport).toBe(42);
  expect(cjs.default).toBe(cjsDefault);
  expect(cjs.namedExport).toBe(42);
  expect(plain.default).toBe(plainDefault);
});
