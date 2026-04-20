const path = require('path');
const fs = require('fs');

const file = path.resolve(__dirname, 'bundle1.js')
const content = fs.readFileSync(file, 'utf-8');

it ('mocked modules should be hoisted', () => {
	const afterTopOfFile = content.indexOf('TOP_OF_FILE');
	expect(afterTopOfFile).toBeGreaterThan(content.lastIndexOf('__webpack_require__.mock'));
})

it ('rs.mock(id, factory) should not turn the module into an async module', () => {
	// `__rspack_async_done` only appears inside `__webpack_require__.a(...)` async wrapper.
	expect(content).not.toContain('__rspack_async_done');
})

it ('rs.mock(id) should not turn the module into an async module', () => {
	const manualMockContent = fs.readFileSync(path.resolve(__dirname, 'bundle2.js'), 'utf-8');
	expect(manualMockContent).not.toContain('__rspack_async_done');
})
