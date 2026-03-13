it('should assign short names to dynamic import chunks', async () => {
	const app = await import('./src/app')
	const lib = await import('./lib/index.js')
	expect(app.value).toBe(1)
	expect(lib.value).toBe(2)
})
