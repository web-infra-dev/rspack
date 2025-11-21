it('should compile', async () => {
	const v1 = await import('./dyn-1').then(m => m.default)
	expect(v1.default).toBe('shared')
})
