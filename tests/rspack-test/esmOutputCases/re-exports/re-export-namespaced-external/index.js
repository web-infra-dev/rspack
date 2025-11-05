export {readFile as r} from 'fs'
// reexport named default
export {default as fs} from 'fs'
// reexport default
export {default} from 'fs'
export * as fsNs from 'fs'

export {readFile} from 'fs'
export { named } from './lib'
export * from './lib2'
export * from 'fs';

import * as fsImportedNs from 'fs'

it('should compile and import success', async () => {
	// make sure fs usedName is Namespace
	expect(fsImportedNs).toBeDefined()
	const {
		fs,
		/* star re-exported */ readFileSync,
		/* renamed re-exported */ r,
		default: defaultExport,
		fsNs,
		readFile
	} = await import(/*webpackIgnore: true*/'./main.mjs')
	expect(fs).toBeDefined()
	expect(defaultExport).toBeDefined()
	expect(fsNs).toBeDefined()
	expect(readFileSync).toBeDefined()
	expect(r).toBeDefined()
	expect(readFile).toBeDefined()
})
