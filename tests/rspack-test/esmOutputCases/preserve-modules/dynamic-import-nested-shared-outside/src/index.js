import { label } from './label'

it('should preserve nested dynamic import modules and shared outside chunks', async () => {
	expect(label).toBe('entry:outside')

	const feature = await import('./feature')

	expect(feature.default()).toBe('feature:util:outside')
	expect(feature.answer).toBe(42)
})
