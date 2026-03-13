it('should dynamically import module with default interop from external', async () => {
	const mod = await import('./wrapper')
	expect(mod.default).toBeDefined()
	expect(mod.readFile).toBeDefined()
	expect(mod.ns).toBeDefined()
})
