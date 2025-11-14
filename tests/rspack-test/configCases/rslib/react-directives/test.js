const path = require('path');
const fs = require('fs');

const testCases = [
	{ name: 'CJS', file: 'bundle0.js' },
	{ name: 'ESM', file: 'bundle1.mjs' },
	{ name: 'ESM (with EsmLibraryPlugin)', file: 'bundle2.mjs' }
];

testCases.forEach(({ name, file }) => {
	it(`should include React directives with double quotes (${name})`, () => {
		const filePath = path.resolve(__dirname, file);
		const content = fs.readFileSync(filePath, 'utf-8');

		expect(content).toContain('"use client"');
	});

	it(`should place directives before actual code (${name})`, () => {
		const filePath = path.resolve(__dirname, file);
		const content = fs.readFileSync(filePath, 'utf-8');
		const clientIndex = content.indexOf('"use client"');

		expect(clientIndex).toBe(0);
	});
});
