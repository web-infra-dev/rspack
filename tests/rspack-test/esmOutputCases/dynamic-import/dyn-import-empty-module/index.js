it('should handle dynamic import of empty module', async () => {
	const mod = await import('./empty');
	expect(typeof mod).toBe('object');
	expect(Object.keys(mod).filter(k => k !== '__esModule')).toHaveLength(0);
});
