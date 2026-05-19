export * from './a';
export * from './b';
export { shared as sharedExplicit } from './a';

it('should handle export star combined with explicit named re-exports', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.fromA).toBe('a');
	expect(mod.fromB).toBe('b');
	expect(mod.shared).toBe('a-version');
	expect(mod.sharedExplicit).toBe('a-version');
});
