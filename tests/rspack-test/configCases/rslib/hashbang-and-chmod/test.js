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

	// File permissions (chmod) are only supported on Unix-like systems (Linux, macOS, BSD, etc.).
	// Skip on Windows and WASM environments where Unix permissions are not supported.
	const onUnix = process.platform !== 'win32' && !process.env.WASM;
	if (onUnix) {
		it(`should set executable permissions (0o755) for files with hashbang (${name})`, () => {
			const filePath = path.resolve(__dirname, file);
			const stats = fs.statSync(filePath);
			const permissions = stats.mode & 0o777;

			expect(permissions).toBe(0o755);
		});
	}
});
