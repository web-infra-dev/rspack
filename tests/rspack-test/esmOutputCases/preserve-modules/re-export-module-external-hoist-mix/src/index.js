import { readFileHoisted, readFileSyncHoisted } from './hoisted'
import { readFileWrapped, readFileSyncWrapped } from './wrapped'

it('should preserve module externals reexports for hoisted and wrapped modules', async () => {
	const fs = await import(/* webpackIgnore: true */ 'fs')

	expect(readFileSyncHoisted).toBe(fs.readFileSync)
	expect(readFileHoisted).toBe(fs.readFile)
	expect(readFileSyncWrapped).toBe(fs.readFileSync)
	expect(readFileWrapped).toBe(fs.readFile)

	const hoisted = await import(/* webpackIgnore: true */ './hoisted.mjs')
	const wrapped = await import(/* webpackIgnore: true */ './wrapped.mjs')

	expect(hoisted.readFileSyncHoisted).toBe(fs.readFileSync)
	expect(hoisted.readFileHoisted).toBe(fs.readFile)
	expect(wrapped.readFileSyncWrapped).toBe(fs.readFileSync)
	expect(wrapped.readFileWrapped).toBe(fs.readFile)
})
