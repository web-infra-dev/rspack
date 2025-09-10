it('should compile', async () => {
	const { default: v1 } = await import(/*webpackChunkName: 'shared'*/'./lib1')

	expect(v1).toBe(1)
})
