it('should compile', async () => {
	const { default: v2 } = await import(/*webpackChunkName: 'shared'*/'./lib2')

	expect(v2).toBe(2)
})
