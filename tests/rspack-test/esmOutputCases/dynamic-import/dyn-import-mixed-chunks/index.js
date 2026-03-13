it('should handle mixed: single-module chunk + multi-module chunk', async () => {
	// solo is alone in its chunk - direct import works
	const solo = await import('./solo.js')
	expect(solo).toHaveProperty('default', 'solo')
	expect(solo).toHaveProperty('value', 100)

	// m1 and m2 share a chunk via magic comment - need namespace objects
	const m1 = await import(/* webpackChunkName: "shared" */'./m1.js')
	const m2 = await import(/* webpackChunkName: "shared" */'./m2.js')

	expect(m1).toHaveProperty('default', 'hello')
	expect(m1).toHaveProperty('value', 1)
	expect(m2).toHaveProperty('default', 'world')
	expect(m2).toHaveProperty('value', 2)
})
