export {default as fs} from 'fs'
export {readFile as r} from 'fs'
export {readFile} from 'fs'
export { named } from './lib'
export * from './lib2'
export * from 'fs';

it('should compile and import success', async () => {
	const {
		fs,
		/* star re-exported */ readFileSync,
		/* renamed re-exported */ r,
		readFile
	} = await import(/*webpackIgnore: true*/'./main.mjs')

	expect(fs).toBeDefined()
	expect(readFileSync).toBeDefined()
	expect(r).toBeDefined()
	expect(readFile).toBeDefined()
})
