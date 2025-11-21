import { demo as otherDemo } from  './other'

export const demo = () => 'demo'

it('should have strict export symbol', async () => {
	const demo = () => 42
	expect(demo()).toBe(42)
	expect(otherDemo()).toBe('otherDemo')
	const allExports = await import(/*webpackIgnore: true*/'./index.mjs')

	expect(allExports).toHaveProperty('demo')
})

