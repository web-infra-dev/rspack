import auto from './auto.ts';
import commonjs from './commonjs/index.ts';
import esm from './esm/index.ts';

it('should use package type for TypeScript module type', () => {
	expect(auto).toBe('auto');
	expect(commonjs).toBe('commonjs');
	expect(esm).toBe('esm');
});
