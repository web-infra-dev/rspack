it('should dynamically import module with mixed external re-exports and local exports', async () => {
	const mod = await import('./wrapper')
	expect(mod.readFile).toBeDefined()
	expect(mod.localValue).toBe(42)
	expect(mod.localFn()).toBe('hello')
})
