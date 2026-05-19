import { value } from './shared'

it('should dynamically import modules from multi-module chunk with mixed re-exports', async () => {
	expect(value).toBe(42)

	const modA = await import('./a')
	expect(modA.value).toBe(42)
	expect(modA.readFile).toBeDefined()

	const modB = await import('./b')
	expect(modB.helper()).toBe(1)
	expect(modB.join).toBeDefined()
})
