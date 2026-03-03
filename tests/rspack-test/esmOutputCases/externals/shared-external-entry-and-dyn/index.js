import { readFile } from 'fs'

export { readFile }

it('should handle external used in both entry and dynamic import', async () => {
	const mod = await import('./async')
	expect(mod.readFile).toBeDefined()
	expect(readFile).toBeDefined()
	expect(mod.readFile).toBe(readFile)
})
