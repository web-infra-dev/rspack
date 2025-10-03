export * from './lib'

it('should re-export esm correctly', async () => {
	const { lib, lib2, lib3 } = await import(/* webpackIgnore: true */ './main.mjs')
	expect(lib).toBe(42)
	expect(lib2).toBe(42)
	expect(lib3).toBe(42)
})
