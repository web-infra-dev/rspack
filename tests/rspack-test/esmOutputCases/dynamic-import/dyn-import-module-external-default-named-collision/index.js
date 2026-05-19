it('should not confuse default and named bindings for module externals in shared chunks', async () => {
	const a = await import(/* webpackChunkName: "shared" */ './a.js')
	const b = await import(/* webpackChunkName: "shared" */ './b.js')
	const fs = await import(/* webpackIgnore: true */ 'fs')

	expect(a.defaultImport).toBe(fs.default)
	expect(b.namedReadFile).toBe(fs.readFile)
	expect(b.namedReadFile).not.toBe(fs.default)
	expect(b.marker).toBe('ok')
})
