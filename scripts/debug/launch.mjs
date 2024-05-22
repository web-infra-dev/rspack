import { fileURLToPath } from "node:url";

import chalk from "chalk";

const { yellow } = chalk;
const __dirname = path.dirname(fileURLToPath(import.meta.url));

export async function launchRspackCli(additionalArgs) {
	let args = [
		"--inspect-brk",
		path.join(__dirname, "../../", "/packages/rspack-cli/bin/rspack"),
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

export async function launchJestWithArgs(additionalArgs) {
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
				...process.env
			},
			cwd: process.cwd()
		}
	];
	console.info(`$ ${yellow("node")} ${args.join(" ")}`);
	await launchDebugger(launch);
}

async function launchDebugger(launchConfig) {
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
	const mapUrl = c =>
		"vscode://fabiospampinato.vscode-debug-launcher/launch?args=" +
		encodeURIComponent(JSON.stringify(c));

	if (process.platform === "win32") {
		return Promise.all(launchConfig.map(c => $`cmd.exe /c start ${mapUrl(c)}`));
	}

	await $`code --open-url ${launchConfig.map(mapUrl)}`;
}
