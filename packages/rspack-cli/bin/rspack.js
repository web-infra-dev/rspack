#!/usr/bin/env node
const nodeModule = require("node:module");

// enable on-disk code caching of all modules loaded by Node.js
// requires Nodejs >= 22.8.0
const { enableCompileCache } = nodeModule;
if (enableCompileCache) {
	try {
		enableCompileCache();
	} catch {
		// ignore errors
	}
}

const { RspackCLI } = require("../dist/index");

async function runCLI() {
	const cli = new RspackCLI();
	await cli.run(process.argv);
}

runCLI();
