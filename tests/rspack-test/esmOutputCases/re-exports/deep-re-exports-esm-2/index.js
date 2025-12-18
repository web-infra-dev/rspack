export * from './lib'

it('should re-export esm correctly', async () => {
	const { lib, lib2, lib3, lib4, readFile, resolve, fs } = await import(/* webpackIgnore: true */ './main.mjs')
	expect(lib).toBe(42)
	expect(lib2).toBe(42)
	expect(lib3).toBe(42)
	expect(lib4).toBe(24)
	expect(typeof readFile).toBe('function')
	expect(typeof resolve).toBe('function')
	expect(fs.readFile).toBe(readFile)
})
