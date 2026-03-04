it('should support dyn import with no conflicting exports', async () => {
	const a = await import(/* webpackChunkName: "shared" */'./a.js')
	const b = await import(/* webpackChunkName: "shared" */'./b.js')

	expect(a).toHaveProperty('foo', 1)
	expect(b).toHaveProperty('bar', 2)
})
