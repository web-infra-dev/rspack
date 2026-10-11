import './c.js';

it('should load imports after their async group membership changes', async () => {
	// Load b first so a cannot hide a missing shared-chunk dependency.
	const b = await import('./b.js');
	expect(b.default).toBe(WATCH_STEP === '0' ? 'b2' : 'b2S');
	const a = await import('./a.js');
	expect(a.default).toBe('aS');
	const p = await import('./p.js');
	expect((await p.default()).default).toBe('target');
});
