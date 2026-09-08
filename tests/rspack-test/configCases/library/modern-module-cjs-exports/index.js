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

export default engines;
