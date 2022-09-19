#!/usr/bin/env node

const cp = require("child_process");

void (function () {
	checkCommander();
	const cli = createCLi();
	cli.parse(process.argv.slice(2));
})();

function checkCommander() {
	const COMMANDER_VERSION = "9.4.0";
	try {
		if (require("commander").version === COMMANDER_VERSION) {
			throw class extends Error {
				tag = "version_error_tag";
			};
		}
	} catch (error) {
		if (error.tag) {
			// - version match failed.
		} else {
			// - require failed
		}
		console.log("installing node_modules");
		cp.exec("pnpm i");
	}
}

function createCLi() {
	const { Command } = require("commander");

	const cli = new Command();
	cli
		.name("install")
		.description("install node dependencies")
		.action(() => {
			cp.exec("pnpm i");
		});

	return cli;
}
