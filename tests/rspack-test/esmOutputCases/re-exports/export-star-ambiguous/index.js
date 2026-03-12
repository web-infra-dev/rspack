// Both a.js and b.js export 'shared'. Rspack resolves the ambiguity by
// picking the first source (a.js) rather than excluding it per strict ES spec.
// Non-overlapping names should still be re-exported normally.
export * from './a';
export * from './b';

it('should handle ambiguous export star without crashing', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.onlyA).toBe('only-a');
	expect(mod.onlyB).toBe('only-b');
	// Rspack picks the first export * source for ambiguous names
	expect(mod.shared).toBe('from-a');
});
