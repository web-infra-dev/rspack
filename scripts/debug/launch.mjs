import chalk from "chalk";
import { fileURLToPath } from "node:url";

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
	const mapUrl = c =>
		"vscode://fabiospampinato.vscode-debug-launcher/launch?args=" +
		encodeURIComponent(JSON.stringify(c));

	if (process.platform === "win32") {
		return Promise.all(
			launchConfig.map(c => $`cmd.exe /c start ${launchConfig.map(mapUrl)}`)
		);
	}

	await $`code --open-url ${launchConfig.map(mapUrl)}`;
}

async function hasCommandCode() {
	let which = process.platform === "win32" ? "where.exe" : "which";
	try {
		let fs = await import("node:fs/promises");
		let { stdout } = await $`${which} node`.quiet();
		await fs.access(stdout.split(/[\n\r]/)[0]);
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
