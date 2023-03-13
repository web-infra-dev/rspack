#!/usr/bin/env node
const { createCLI } = require("./scripts/cmd.js");

/// use `./x --help` to get more information..
void (function () {
	const cli = createCLI();
	cli.parse(process.argv);
})();
