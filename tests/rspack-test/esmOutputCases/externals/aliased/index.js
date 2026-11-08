export * from 'external'

it('should have correct exports', async () => {
  const { resolve } = await import(/*webpackIgnore: true*/'./main.mjs')

	const { resolve: nodeResolve } = await import(/* webpackIgnore: true */ 'node:path')
	expect(resolve).toBe(nodeResolve)
});
