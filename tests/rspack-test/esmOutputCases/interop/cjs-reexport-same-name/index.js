import { value as valueA, unique_a } from './a.cjs';
import { value as valueB, unique_b } from './b.cjs';

export { valueA, valueB, unique_a, unique_b };

it('should correctly handle two CJS modules with same-named exports', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.valueA).toBe('from-a');
	expect(mod.valueB).toBe('from-b');
	expect(mod.unique_a).toBe(100);
	expect(mod.unique_b).toBe(200);
});
