import { value } from './shared'

it('should dynamically import module that re-exports from cross-chunk module', async () => {
	expect(value).toBe(42)

	const mod = await import('./wrapper')
	expect(mod.value).toBe(42)
	expect(mod.helper()).toBe(1)
})
