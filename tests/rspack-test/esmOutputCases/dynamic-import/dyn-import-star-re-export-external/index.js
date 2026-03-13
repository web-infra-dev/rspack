it('should dynamically import module with star re-export from external', async () => {
	const mod = await import('./wrapper')
	expect(mod.readFile).toBeDefined()
})
