export * from './lib'

it('should re-export esm correctly', async () => {
	const { lib, lib2, lib3, lib4, path, fs } = await import(/* webpackIgnore: true */ './main.mjs')
	expect(lib).toBe(42)
	expect(lib2).toBe(42)
	expect(lib3).toBe(42)
	expect(lib4).toBe(24)
	expect(typeof path.resolve).toBe('function')
	expect(typeof fs.readFile).toBe('function')
})
