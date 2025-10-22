export const demo = () => 'otherDemo'

{
	const demo = () => 42
	console.log.bind(demo)
}

it('should have strict export symbol', async () => {
	const allExports = await import(/*webpackIgnore: true*/'./other.mjs')

	expect(allExports).toHaveProperty('demo')
})
