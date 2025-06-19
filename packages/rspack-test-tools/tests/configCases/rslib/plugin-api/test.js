const path = require('path');
const fs = require('fs');

const file = path.resolve(__dirname, 'bundle0.js')
const content = fs.readFileSync(file, 'utf-8');

it ('`require.cache` should be not handled by APIPlugin', () => {
	expect(content).toContain('console.log(require.cache)')
})
