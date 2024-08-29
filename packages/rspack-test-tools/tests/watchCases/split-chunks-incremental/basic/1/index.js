it('should compile', async () => {
	const [v1, v2] = await Promise.all([
		import('./dyn-1').then(m => m.default),
		import('./dyn-2').then(m => m.default)
	])
	expect(v1).toBe('v')
	expect(v2).toBe(2)
})
