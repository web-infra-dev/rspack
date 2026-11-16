const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

const HASH = /"sha256-[A-Za-z0-9+/]{43}="/g;

// The runtime removes each script once it loads, so record them as they are added.
const scripts = [];
const appendChild = document.head.appendChild.bind(document.head);
document.head.appendChild = (node) => {
	if (node.tagName === 'SCRIPT') scripts.push(node);
	return appendChild(node);
};

function fileOf(script) {
	return path.basename(new URL(script.src).pathname);
}

function countHashes(file) {
	return fs.readFileSync(path.join(__dirname, file), 'utf-8').match(HASH)?.length ?? 0;
}

function expectAllVerified(count) {
	expect(scripts.length).toBe(count);
	for (const script of scripts) {
		const source = fs.readFileSync(path.join(__dirname, fileOf(script)));
		const integrity = `sha256-${crypto.createHash('sha256').update(source).digest('base64')}`;
		expect(script.integrity).toBe(integrity);
	}
}

// No provider registers shared-lib, so ensuring feature's chunk makes the runtime
// load the fallback's chunk before feature's code has run. The chunk that requests
// feature must already provide the fallback's hash.
it('should verify a shared module fallback loaded while its chunk is ensured', async () => {
	const { value } = await import(/* webpackChunkName: "feature" */ './feature');
	expect(value).toBe(42);
	expectAllVerified(2);
});
