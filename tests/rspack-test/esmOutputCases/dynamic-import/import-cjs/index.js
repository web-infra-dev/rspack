it('should support dynamic import', async () => {
	const mod = await import('./dynamic')
	expect(mod.value).toBe(42)
})

export {}
