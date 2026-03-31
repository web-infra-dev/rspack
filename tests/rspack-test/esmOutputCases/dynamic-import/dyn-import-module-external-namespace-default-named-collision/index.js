it('should keep named reexports distinct from default imports in module-external namespace objects', async () => {
	const a = await import(/* webpackChunkName: "shared" */ './a.js')
	const b = await import(/* webpackChunkName: "shared" */ './b.js')

	expect(a.default).toBe('default-foo')
	expect(a.shared).toBe('a')
	expect(b.shared).toBe('b')
	expect(b.namedFoo).toBe('named-foo')
	expect(b.namedFoo).not.toBe('default-foo')
})
