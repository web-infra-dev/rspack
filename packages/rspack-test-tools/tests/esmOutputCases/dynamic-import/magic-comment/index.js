it('should support dynamic import', async () => {
	const value = await import(/* webpackChunkName: "shared" */'./m1.js')
	const value2 = await import(/* webpackChunkName: "shared" */'./m2.js')

	expect(value).tohaveProperty('default', 42)
	expect(value).tohaveProperty('value', 1)
	expect(value2).tohaveProperty('default', 42)
	expect(value2).tohaveProperty('value', 2)
})

