it('should dynamically import module that re-exports from external', async () => {
	const mod = await import('./wrapper')
	expect(mod.readFile).toBeDefined()
	expect(mod.fsDefault).toBeDefined()
})
