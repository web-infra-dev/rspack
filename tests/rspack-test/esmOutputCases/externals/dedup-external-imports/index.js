import { readFile } from './lib'
import { ns } from './lib2'

it('should not have duplicate import identifiers', async () => {
	expect(readFile).toBeDefined()
	expect(ns).toBeDefined()
	expect(ns.readFile).toBe(readFile)
})
