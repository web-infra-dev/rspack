it('should keep namespace when both conflicting exports are used', async () => {
	// Both modules export `value` and both are used — real conflict.
	// Namespace objects are required to disambiguate.
	const { value: aValue, foo } = await import(/* webpackChunkName: "shared" */'./a.js')
	const { value: bValue, bar } = await import(/* webpackChunkName: "shared" */'./b.js')

	expect(aValue).toBe(1)
	expect(bValue).toBe(2)
	expect(foo).toBe('a-foo')
	expect(bar).toBe('b-bar')
})
