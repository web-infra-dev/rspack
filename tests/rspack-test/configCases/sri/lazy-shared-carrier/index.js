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

function expectVerified(expectedNames) {
	expect(scripts.map((script) => fileOf(script).split('.')[0]).sort()).toEqual(expectedNames);
	for (const script of scripts) {
		const source = fs.readFileSync(path.join(__dirname, fileOf(script)));
		const integrity = `sha256-${crypto.createHash('sha256').update(source).digest('base64')}`;
		expect(script.integrity).toBe(integrity);
	}
}

function chunkFile(name) {
	return fs.readdirSync(__dirname).find((file) => file.split('.')[0] === name);
}

it('should verify a group whose modules splitChunks moved into shared chunks', async () => {
	const boot = await import(/* webpackChunkName: "boot" */ './boot');
	const menu = await import(/* webpackChunkName: "menu" */ './menu');
	// Both boot and menu load fork, so it has two parents, and only boot loads util.
	expect(boot.loadFork).toBe(menu.loadFork);
	const fork = await menu.loadFork();
	expect(fork.value).toBe('util:ab');
	await fork.load();
	expect(boot.value).toBe('util:boot');
	expect((await import(/* webpackChunkName: "other" */ './other')).other).toBe('autil:ab');
	expect((await import(/* webpackChunkName: "other2" */ './other2')).other2).toBe('butil:ab');
	expectVerified(['boot', 'leaf', 'menu', 'other', 'other2', 'shared', 'util']);
});

it('should carry hashes in the chunk that makes the import() call', () => {
	// fork loads [shared, util], and boot also loads util. The import() of leaf is in
	// fork.js, which splitChunks moved to shared, so shared carries leaf's hash and
	// loading boot does not download it.
	expect(countHashes(chunkFile('util'))).toBe(0);
	expect(countHashes(chunkFile('shared'))).toBe(1);
});
