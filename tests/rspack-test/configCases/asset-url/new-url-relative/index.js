import fs from 'fs'
import path from 'path'

it('should generate a static URL without JavaScript output', () => {
	const file = fs.readFileSync(path.resolve(import.meta.dirname, `./test-${INDEX}.mjs`), 'utf-8');
	expect(file).toContain('new URL(');
	expect(file).toContain('import.meta.url');
	expect(file.match(/asset\/static-asset\.js/g)).toHaveLength(1);
})

it('should keep JavaScript output when the asset is also imported', () => {
	const file = fs.readFileSync(path.resolve(import.meta.dirname, `./mixed-${INDEX}.mjs`), 'utf-8');
	expect(file).toContain('new URL(');
	expect(file.match(/asset\/static-mixed-asset\.js/g)).toHaveLength(2);
})

it('should keep JavaScript output for other URL modes', () => {
	const file = fs.readFileSync(path.resolve(import.meta.dirname, `./normal-${INDEX}.mjs`), 'utf-8');
	expect(file).toContain('new URL(');
	expect(file.match(/asset\/static-normal-asset\.js/g)).toHaveLength(1);
})
