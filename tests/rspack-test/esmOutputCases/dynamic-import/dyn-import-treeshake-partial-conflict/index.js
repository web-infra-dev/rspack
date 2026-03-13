it('should skip namespace when tree-shaking removes one side of a conflict', async () => {
	// Both modules export `value`, but only a.value is used.
	// b.value is tree-shaken, so the conflict disappears.
	const { value, foo } = await import(/* webpackChunkName: "shared" */'./a.js')
	const { bar } = await import(/* webpackChunkName: "shared" */'./b.js')

	expect(value).toBe(1)
	expect(foo).toBe('a-foo')
	expect(bar).toBe('b-bar')
})
