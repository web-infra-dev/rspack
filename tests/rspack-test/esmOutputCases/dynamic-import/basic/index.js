it('should support dynamic import', async () => {
	const value = await import('./dynamic')
	expect(value.default).toBe(42)
})

