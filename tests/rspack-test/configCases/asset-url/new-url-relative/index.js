import fs from 'fs'
import path from 'path'

it('should have correct url', () => {
	const file = fs.readFileSync(path.resolve(import.meta.dirname, `./test-${INDEX}.mjs`), 'utf-8');
	expect(file).toContain('new URL(');
	expect(file).toContain('import.meta.url');
})
