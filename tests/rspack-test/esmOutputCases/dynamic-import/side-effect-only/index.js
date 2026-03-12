it('should handle dynamic import of side-effect-only module', async () => {
	globalThis.__sideEffectExecuted = false;
	await import('./side-effect');
	expect(globalThis.__sideEffectExecuted).toBe(true);
});
