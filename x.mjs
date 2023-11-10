#!/usr/bin/env zx

import "zx/globals";
import chalk from "chalk";
import { Command } from "commander";
import { version_handler } from "./scripts/release/version.mjs";
import { publish_handler } from "./scripts/release/publish.mjs";
const { yellow } = chalk;

process.env.FORCE_COLOR = 3; // Fix zx losing color output in subprocesses

const program = new Command();

program
	.name("Rspack Development CLI")
	.description("CLI for development of Rspack")
	.showHelpAfterError(true)
	.showSuggestionAfterError(true);

// x ready
program
	.command("ready")
	.alias("r")
	.description("ready to create a pull request, build and run all tests")
	.action(async function () {
		await $`cargo check`;
		await $`cargo lint`;
		await $`cargo test`;
		await $`pnpm install`;
		await $`pnpm run build:cli:release`;
		await $`pnpm run test:example`;
		await $`pnpm run test:unit`;
		console.log(chalk.green("All passed."));
	});

// x install
program
	.command("install")
	.alias("i")
	.description("install all dependencies")
	.action(async function () {
		await $`pnpm install`;
	});

// x clean
let cleanCommand = program
	.command("clean")
	.description("clean target/ directory");

// x clean all
cleanCommand.command("all").action(async function () {
	await $`./x clean rust`;
});

// x clean rust
cleanCommand
	.command("rust")
	.description("clean target/ directory")
	.action(async function () {
		await $`cargo clean`;
	});

// x build
const buildCommand = program.command("build").alias("b").description("build");
const watchCommand = program.command("watch").alias("w").description("watch");

buildCommand
	.option("-a", "build all")
	.option("-b", "build rust binding")
	.option("-j", "build js packages")
	.option("-r", "release")
	.action(async function ({ a, b = a, j = a, r }) {
		let mode = r ? "release" : "debug";
		b && (await $`pnpm --filter @rspack/binding build:${mode}`);
		j && (await $`pnpm --filter "@rspack/*" build`);
	});

watchCommand
	.option("-a", "watch all")
	.option("-b", "watch rust binding")
	.option("-j", "watch js packages")
	.option("-r", "release")
	.action(async function ({ a, b = a, j = a, r }) {
		let mode = r ? "release" : "debug";
		b && (await $`pnpm --filter @rspack/binding watch:${mode}`);
		j && (await $`pnpm --filter "@rspack/*" watch`);
	});

// x build binding
buildCommand
	.command("binding")
	.description("build rust binding")
	.action(async function () {
		await $`pnpm --filter @rspack/binding build:debug`;
	});

// x build js
buildCommand
	.command("js")
	.description("build js packages")
	.action(async function () {
		await $`pnpm --filter "@rspack/*" build`;
	});

// x test
const testCommand = program.command("test").alias("t").description("test");

// x test rust
testCommand
	.command("rust")
	.description("run cargo tests")
	.action(async function () {
		await $`cargo test`;
	});

// x test example
testCommand
	.command("example")
	.description("build examples")
	.action(async function () {
		await $`pnpm --filter "example-*" build`;
	});

// x test unit
testCommand
	.command("unit")
	.description("run all unit tests")
	.action(async function () {
		await $`./x build js`;
		await $`pnpm --filter "@rspack/*" test`;
	});

// x test ci
testCommand
	.command("ci")
	.description("run tests for ci")
	.action(async function () {
		await $`./x test example`;
		await $`./x test unit`;
	});
// x test webpack
testCommand
	.command("webpack")
	.description("run webpack test suites")
	.action(async function () {
		await $`pnpm --filter "webpack-test" test`;
	});

// x rspack / x rs
const rspackCommand = program.command("rspack").alias("rs").description(`
  $ x rspack -- [your-rspack-cli-args...]
  $ x rspack --debug -- build
  $ x rs -d -- build
  $ x rsd -- build
`);

rspackCommand
	.option("-d, --debug", "Launch debugger in VSCode")
	.action(async function ({ debug }) {
		if (!debug) {
			await $`npx rspack ${getVariadicArgs().toString()}`;
			return;
		}
		await launchRspackCli(getVariadicArgs());
	});

// x rsd
program
	.command("rspack-debug")
	.alias("rsd")
	.description("Alias for `x rspack --debug`")
	.action(async function () {
		await launchRspackCli(getVariadicArgs());
	});

// x jest / x j
const jestCommand = program.command("jest").alias("j").description(`
  $ x jest -- [your-jest-args...]
  $ x jest --debug -- -t <test-name-pattern>
  $ x j -d -- [test-path-pattern]
  $ x jd -- [your-jest-args...]
`);

jestCommand
	.option("-d, --debug", "Launch debugger in VSCode")
	.action(async ({ debug }) => {
		if (!debug) {
			await $`npx jest ${getVariadicArgs().toString()}`;
			return;
		}
		await launchJestWithArgs(getVariadicArgs());
	});

// x jd
program
	.command("jest-debug")
	.alias("jd")
	.description("Alias for `x jest --debug`")
	.action(async function () {
		await launchJestWithArgs(getVariadicArgs());
	});

program
	.command("version")
	.argument("<bump_version>", "bump version to (major|minor|patch|snapshot)")
	.description("bump version")
	.action(version_handler);

program
	.command("publish")
	.argument("<mode>", "publish mode (snapshot|stable)")
	.requiredOption("--tag <char>", "publish tag")
	.option(
		"--dry-run",
		"Does everything a publish would do except actually publishing to the registry"
	)
	.option("--no-dry-run", "negative dry-run")
	.option("--push-tags", "push tags to github")
	.option("--no-push-tags", "don't push tags to github")
	.description("publish package after version bump")
	.action(publish_handler);
let argv = process.argv.slice(2); // remove the `node` and script call
if (argv[0] && /x.mjs/.test(argv[0])) {
	// Called from `zx x.mjs`
	argv = argv.slice(1);
}
program.parse(argv, { from: "user" });

async function launchRspackCli(additionalArgs) {
	let args = [
		"--inspect-brk",
		"${workspaceFolder}/packages/rspack-cli/bin/rspack",
		...additionalArgs
	];
	let launch = [
		{
			name: "rust",
			type: "lldb",
			request: "launch",
			sourceLanguages: ["rust"],
			program: "node",
			args,
			env: process.env,
			cwd: process.cwd()
		}
	];
	console.info(`$ ${yellow("node")} ${args.join(" ")}`);
	await launchDebugger(launch);
}

async function launchJestWithArgs(additionalArgs) {
	let args = [
		"--inspect-brk",
		"--expose-gc",
		"--max-old-space-size=8192",
		"--experimental-vm-modules",
		"${workspaceFolder}/node_modules/.bin/jest",
		"--runInBand",
		"--logHeapUsage"
	];
	if (additionalArgs) {
		args.push(...additionalArgs);
	}
	let launch = [
		{
			name: "rust",
			type: "lldb",
			request: "launch",
			sourceLanguages: ["rust"],
			program: "node",
			args,
			env: {
				NO_COLOR: JSON.stringify(1),
				RSPACK_DEP_WARNINGS: JSON.stringify(false),
				...process.env
			},
			cwd: process.cwd()
		}
	];
	console.info(`$ ${yellow("node")} ${args.join(" ")}`);
	await launchDebugger(launch);
}

async function launchDebugger(launchConfig) {
	if (!(await hasCommandCode()) || !(await hasLaunchExtensionInstalled())) {
		return;
	}
	launchConfig = [
		...launchConfig,
		{
			name: "node",
			port: 9229,
			request: "attach",
			skipFiles: ["<node_internals>/**"],
			sourceMaps: true,
			continueOnAttach: true,
			type: "node"
		}
	];
	console.info(yellow("Initializing VSCode debugger..."));
	await $`code --open-url ${launchConfig.map(
		c =>
			"vscode://fabiospampinato.vscode-debug-launcher/launch?args=" +
			JSON.stringify(c)
	)}`;
}

// Get args after `--`
function getVariadicArgs() {
	let idx = argv.findIndex(c => c === "--");
	let args = idx === -1 ? [] : argv.slice(idx + 1);
	Object.assign(args, {
		toString() {
			return this.join(" ");
		}
	});
	return args;
}

async function hasCommandCode() {
	let which = process.platform === "win32" ? "where.exe" : "which";
	try {
		let fs = await import("node:fs/promises");
		let { stdout } = await $`${which} node`.quiet();
		await fs.access(stdout.replace(/[\n\r]/g, ""));
		return true;
	} catch (p) {
		console.error(
			new Error(p.stderr || p.message, {
				cause:
					"Only Vscode has been supported by now. Did you forget to install 'code' command?"
			})
		);
		return false;
	}
}

async function hasLaunchExtensionInstalled() {
	try {
		let { stdout, stderr } = await $`code --list-extensions`.quiet();
		if (stderr) {
			console.error(stderr);
			return false;
		}
		return stdout?.includes("fabiospampinato.vscode-debug-launcher");
	} catch (p) {
		console.error(
			new Error(p.stderr || p.message, {
				cause:
					"VSCode extension `fabiospampinato.vscode-debug-launcher` is required. https://marketplace.visualstudio.com/items?itemName=fabiospampinato.vscode-debug-launcher"
			})
		);
		return false;
	}
}
