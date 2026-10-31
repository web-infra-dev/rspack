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

function countHashes(file) {
	return fs.readFileSync(path.join(__dirname, file), 'utf-8').match(HASH)?.length ?? 0;
}

it('should load each chunk with the hash its parent carried', async () => {
	const { deeper } = await import(/* webpackChunkName: "a" */ './a');
	const { deepest } = await deeper();
	const { value } = await deepest();
	expect(value).toBe('c');

	expect(scripts.map((script) => path.basename(script.src)).sort()).toEqual([
		'a.js',
		'b.js',
		'c.js',
	]);
	for (const script of scripts) {
		const source = fs.readFileSync(path.join(__dirname, path.basename(script.src)));
		const integrity = `sha256-${crypto.createHash('sha256').update(source).digest('base64')}`;
		expect(script.integrity).toBe(integrity);
	}
});

it('should keep each hash in the chunk that loads it', () => {
	expect(countHashes('bundle0.js')).toBe(1);
	expect(countHashes('a.js')).toBe(1);
	expect(countHashes('b.js')).toBe(1);
	expect(countHashes('c.js')).toBe(0);
});
