it('should handle three modules in one chunk with conflicting exports', async () => {
	const a = await import(/* webpackChunkName: "merged" */'./a.js')
	const b = await import(/* webpackChunkName: "merged" */'./b.js')
	const c = await import(/* webpackChunkName: "merged" */'./c.js')

	expect(a).toHaveProperty('default', 'a')
	expect(a).toHaveProperty('name', 'module-a')
	expect(a).toHaveProperty('id', 1)

	expect(b).toHaveProperty('default', 'b')
	expect(b).toHaveProperty('name', 'module-b')
	expect(b).toHaveProperty('id', 2)

	expect(c).toHaveProperty('default', 'c')
	expect(c).toHaveProperty('name', 'module-c')
	expect(c).toHaveProperty('id', 3)
})
