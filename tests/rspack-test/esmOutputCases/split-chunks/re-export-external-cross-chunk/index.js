import { readFile, join } from './wrapper'

export { readFile, join }

it('should have correct exports from cross-chunk external re-exports', async () => {
	const mod = await import(/*webpackIgnore: true*/ './main.mjs')
	expect(mod.readFile).toBeDefined()
	expect(mod.join).toBeDefined()
})
