export default 'vendor2'

it('should not contain vendor1 vendor2 in file', () => {
	const path = require('path')
	const fs = require('fs')

	const code = fs.readFileSync(path.resolve(__dirname, './main.js'), 'utf-8')
	expect(code).not.toContain('"./vendor1.js":')
	expect(code).not.toContain('"./vendor2.js":')
})
