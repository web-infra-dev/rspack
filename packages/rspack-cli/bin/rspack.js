#!/usr/bin/env node
const { RspackCLI } = require("../dist/index");

async function runCLI() {
	const cli = new RspackCLI();
	await cli.run(process.argv);
}

runCLI();
