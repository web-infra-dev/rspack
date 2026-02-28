export { f } from './module'

it('should handle aliased external', async () => {
	const {f, w} = await import(/*webpackIgnore: true*/'./main.mjs')

	expect(f).toBe(__non_webpack_require__('fs').readFile)
	expect(w).toBe(__non_webpack_require__('fs').writeFile)
})

export { writeFile as w } from 'fs'
