export * from 'external'

it('should have correct exports', async () => {
  const { resolve } = await import(/*webpackIgnore: true*/'./main.mjs')

	const { resolve: nodeResolve } = __non_webpack_require__('path')
	expect(resolve).toBe(nodeResolve)
});
