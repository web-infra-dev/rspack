export { f } from './module'

it('should handle aliased external', async () => {
	const {f, w} = await import(/*webpackIgnore: true*/'./main.mjs')
	const fs = await import(/* webpackIgnore: true */ 'node:fs')

	expect(f).toBe(fs.readFile)
	expect(w).toBe(fs.writeFile)
})

export { writeFile as w } from 'fs'
