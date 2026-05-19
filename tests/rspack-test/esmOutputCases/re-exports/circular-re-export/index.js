export { aValue, bValue } from './a.js';

it('should handle circular re-exports correctly', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.aValue).toBe('a');
	expect(mod.bValue).toBe('b');
});
