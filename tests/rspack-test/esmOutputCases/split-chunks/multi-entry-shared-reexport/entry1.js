import { a } from './lib'

export { a }

it('should import different exports from shared lib chunk', async () => {
	const e1 = await import(/*webpackIgnore: true*/ './main.mjs')
	const e2 = await import(/*webpackIgnore: true*/ './entry2.mjs')
	expect(e1.a()).toBe(42)
	expect(e2.b()).toBe(42)
})
