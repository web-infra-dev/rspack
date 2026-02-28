import fs from 'fs'
import path from 'path'

it('should contains default export for json module', async () => {
	const json = fs.readFileSync(path.resolve(import.meta.dirname, './json.mjs'), 'utf-8');
	expect(json).toContain("export default ");
	const jsonModule = await import(/*webpackIgnore: true*/'./json.mjs');
	expect(jsonModule.default).toEqual({ foo: 'bar' });
})
