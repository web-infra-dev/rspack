it('should load a shared chunk after it joins another async group', async () => {
	// Load b first so a cannot hide a missing shared-chunk dependency.
	const b = await import('./b.js');
	expect(b.default).toBe(WATCH_STEP === '0' ? 'b2' : 'b2S');
	const a = await import('./a.js');
	expect(a.default).toBe('aS');
});
