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

// shared holds the runtime, and main, which depends on it, loads a with its CSS.
it('should verify the chunks of an entry that depends on another entry', async () => {
	expect(shared).toBe('shared');
	const a = await import(/* webpackChunkName: "a" */ './a');
	expect(a.value).toBe('a');
	expectVerified(['a']);
	const link = document.querySelector('link[href*="a.css"]');
	const css = fs.readFileSync(path.join(__dirname, 'a.css'));
	expect(link.integrity).toBe(`sha256-${crypto.createHash('sha256').update(css).digest('base64')}`);
});
