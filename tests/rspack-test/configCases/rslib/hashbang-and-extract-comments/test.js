const path = require('path');
const fs = require('fs');

const testCases = [
	{ name: 'CJS', file: 'bundle0.js' },
	{ name: 'ESM', file: 'bundle1.mjs' },
	{ name: 'ESM (with EsmLibraryPlugin)', file: 'bundle2.mjs' }
];

testCases.forEach(({ name, file }) => {
	it(`should include hashbang at the first line (${name})`, () => {
		const filePath = path.resolve(__dirname, file);
		const content = fs.readFileSync(filePath, 'utf-8');

		expect(content.startsWith('#!/usr/bin/env node\n')).toBe(true);
	});
});
