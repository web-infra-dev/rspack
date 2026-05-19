it('should support dynamic import', async () => {
	const mod = await import('./dynamic')
	const mod2 = await import('./esm')
	expect(mod.value).toBe(42)
	expect(mod2.value).toBe(42)
})

export {}
