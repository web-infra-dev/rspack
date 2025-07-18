const path = require('path');
const fs = require('fs');

const file = path.resolve(__dirname, 'bundle1.js')
const content = fs.readFileSync(file, 'utf-8');

it ('mocked modules should be hoisted', () => {
	const afterTopOfFile = content.indexOf('TOP_OF_FILE');
	expect(afterTopOfFile).toBeGreaterThan(content.lastIndexOf('__webpack_require__.mock'));
})
