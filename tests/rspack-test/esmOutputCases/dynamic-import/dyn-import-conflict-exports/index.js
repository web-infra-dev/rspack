import { value as aValue } from './a'
import { value as bValue } from './b'

it('should handle conflicting exports from multi-module chunk', async () => {
	expect(aValue).toBe(1)
	expect(bValue).toBe(2)

	const modA = await import('./a')
	const modB = await import('./b')

	expect(modA.value).toBe(1)
	expect(modB.value).toBe(2)
})
