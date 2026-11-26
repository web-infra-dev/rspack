import './setup.js';
import './babel-esm-marker.cjs';
import './typescript-esm-marker.cjs';
import './data-descriptor.cjs';
import './getter-descriptor.cjs';
import './accessor-property.cjs';
import './impure-value.cjs';
import './shadowed-object.cjs';
import './module-exports-target.cjs';
import './non-plain-module-exports.cjs';
import './sequence-exports.cjs';
import './var-init-define-property.cjs';
import './impure-sequence.cjs';

afterAll(() => {
  delete globalThis.__CJS_DEFINE_PROPERTY_IMPURE_VALUE_COUNT__;
  delete globalThis.__CJS_DEFINE_PROPERTY_SHADOWED_COUNT__;
  delete globalThis.__CJS_DEFINE_PROPERTY_IMPURE_SEQUENCE__;
  delete globalThis.__recordCjsDefinePropertyImpureValue;
});

const readBundle = () => require('fs').readFileSync(__filename, 'utf-8');

it('drops modules whose defineProperty calls only add data properties to exports', () => {
  const source = readBundle();
  const sentinels = [
    ['BABEL', 'ESM', 'MARKER', 'SENTINEL'],
    ['TYPESCRIPT', 'ESM', 'MARKER', 'SENTINEL'],
    ['DATA', 'DESCRIPTOR', 'SENTINEL'],
    ['MODULE', 'EXPORTS', 'TARGET', 'SENTINEL'],
    ['SEQUENCE', 'EXPORTS', 'SENTINEL'],
    ['VAR', 'INIT', 'DEFINE', 'PROPERTY', 'SENTINEL'],
  ];

  for (const parts of sentinels) {
    expect(source).not.toContain(parts.join('_'));
  }
});

it('keeps defineProperty calls with observable effects', () => {
  expect(globalThis.__CJS_DEFINE_PROPERTY_IMPURE_VALUE_COUNT__).toBe(1);
  expect(globalThis.__CJS_DEFINE_PROPERTY_SHADOWED_COUNT__).toBe(1);
  expect(globalThis.__CJS_DEFINE_PROPERTY_IMPURE_SEQUENCE__).toBe(true);
});

it('keeps accessor descriptors and non-plain module.exports targets', () => {
  const source = readBundle();
  const sentinels = [
    ['GETTER', 'DESCRIPTOR', 'SENTINEL'],
    ['ACCESSOR', 'PROPERTY', 'SENTINEL'],
    ['NON', 'PLAIN', 'MODULE', 'EXPORTS', 'SENTINEL'],
    ['IMPURE', 'SEQUENCE', 'SENTINEL'],
  ];

  for (const parts of sentinels) {
    expect(source).toContain(parts.join('_'));
  }
});
