const fs = require('fs');
const path = require('path');

it('should keep async startup safe in non-federation entry runtime', () => {
	const runtimeFile = path.join(__dirname, 'plainRuntime.js');
	const entryFile = path.join(__dirname, 'plain.js');

	const runtime = fs.readFileSync(runtimeFile, 'utf-8');
	const entry = fs.readFileSync(entryFile, 'utf-8');

	expect(entry).not.toMatch(/= __webpack_require__\.X\(\);/);
	expect(runtime).not.toContain('var prevStartup');
	expect(__webpack_require__.federation).toBeUndefined();
});
