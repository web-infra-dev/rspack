it('should use package name instead of dist for node_modules short name', async () => {
	const lib = await import('my-lib')
	expect(lib.value).toBe(42)
})
