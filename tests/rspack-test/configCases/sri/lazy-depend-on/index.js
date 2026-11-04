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

import { shared } from './shared';

// main depends on shared, which makes main's chunk group a child of shared's.
it('should start an entry that depends on another entry', () => {
	expect(shared).toBe('shared');
});

it('should not carry hashes for entry chunks', () => {
	expect(countHashes('shared.js')).toBe(0);
	expect(fs.readFileSync(path.join(__dirname, 'shared.js'), 'utf-8')).not.toContain('childHashes');
});
