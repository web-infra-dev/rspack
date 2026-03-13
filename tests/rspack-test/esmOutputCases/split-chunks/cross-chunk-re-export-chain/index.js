import { value, helper } from './middle'

export { value, helper }

it('should have correct exports through cross-chunk re-export chain', async () => {
	const mod = await import(/*webpackIgnore: true*/ './main.mjs')
	expect(mod.value).toBe(42)
	expect(mod.helper()).toBe(1)
})
