it('should support dyn import with no conflicting exports', async () => {
	const { foo, fooFn } = await import(/* webpackChunkName: "shared" */'./a.js')
	const { bar, barFn } = await import(/* webpackChunkName: "shared" */'./b.js')

	expect(foo).toBe(1)
	expect(bar).toBe(2)

	expect(fooFn()).toBe(1)
	expect(barFn()).toBe(2)
})
