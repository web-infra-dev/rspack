const path = require('path');
const fs = require('fs');

const file = path.resolve(__dirname, 'bundle0.js')
const content = fs.readFileSync(file, 'utf-8');

it ('some expressions should not be handled by APIPlugin', () => {
	expect(content).toContain('console.log(require.cache)')
	expect(content).toContain('console.log(require.extensions)')
	expect(content).toContain('console.log(require.config)')
	expect(content).toContain('console.log(require.version)')
	expect(content).toContain('console.log(require.include)')
	expect(content).toContain('console.log(require.onError)')
	expect(content).toContain('module.children = module.children.filter((item) => item.filename !== path)')
	expect(content).not.toContain('__webpack_require__')
})
