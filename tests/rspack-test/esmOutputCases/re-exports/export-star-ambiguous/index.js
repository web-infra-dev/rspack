// Both a.js and b.js export 'shared'. Per ES spec, ambiguous star exports
// should be excluded. The non-overlapping names should still be re-exported.
export * from './a';
export * from './b';

it('should handle ambiguous export star (shared name excluded per spec)', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.onlyA).toBe('only-a');
	expect(mod.onlyB).toBe('only-b');
});
