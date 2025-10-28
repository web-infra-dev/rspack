import { demo as local } from './demo'

it('should preserve modules', () => {
	const local = () => 2
	expect(local()).toBe(2)
})

it('should have strict export symbol', async () => {
	const allExports = await import(/*webpackIgnore: true*/'./index.mjs')

	expect(allExports).toHaveProperty('local')
})

export { local }
