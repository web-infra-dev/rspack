export {default as fs} from 'fs'
export {readFile as r} from 'fs'
export {readFile} from 'fs'
export { named } from './lib'
export * from './lib2'
export * from 'fs';

// should wrap entry because of module object access
console.log.bind(module);

it('should compile and import success', async () => {
	const {
		fs,
		/* renamed re-exported */ r,
		readFile
	} = await import(/*webpackIgnore: true*/'./main.mjs')

	expect(fs).toBeDefined()
	expect(r).toBeDefined()
	expect(readFile).toBeDefined()
})
