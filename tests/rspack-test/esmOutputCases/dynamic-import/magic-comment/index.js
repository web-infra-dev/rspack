import fs from 'node:fs'
import path from 'node:path'

it('should support dynamic import', async () => {
	const shared = path.resolve(__dirname, 'shared.mjs')
	const content = fs.readFileSync(shared, 'utf-8')

	expect(content).toContain('// ./m1.js')
	expect(content).toContain('// ./m2.js')

	const value = await import(/* webpackChunkName: "shared" */'./m1.js')
	const value2 = await import(/* webpackChunkName: "shared" */'./m2.js')

	expect(value).toHaveProperty('default', 42)
	expect(value).toHaveProperty('value', 1)
	expect(value2).toHaveProperty('default', 42)
	expect(value2).toHaveProperty('value', 2)
})
