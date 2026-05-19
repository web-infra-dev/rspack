const { positionals } = require("node:util").parseArgs({
	args: process.argv.slice(2),
	options: {
		profile: {
			type: "string"
		}
	},
	strict: true,
	allowPositionals: true
});

const { spawn } = require("node:child_process");

const CARGO_SAFELY_EXIT_CODE = 0;

build().then((value) => {
	// Regarding cargo's non-zero exit code as an error.
	if (value !== CARGO_SAFELY_EXIT_CODE) {
		process.exit(value);
	}
}).catch(err => {
	console.error(err);
	process.exit(1);
});

async function build() {
	return new Promise((resolve, reject) => {
		const args = [
			"build",
			"--platform",
			"--no-js",
			"--no-dts-header",
		];
		const features = [];
		const envs = { ...process.env };

		if (!process.env.DISABLE_PLUGIN) {
			args.push("--no-default-features");
			features.push("plugin");
		}
		args.push("--no-dts-cache");
		if (features.length) {
			args.push(`--features ${features.join(",")}`);
		}

		if (positionals.length > 0) {
			// napi need `--` to separate options and positional arguments.
			args.push("--");
			args.push(...positionals);
		}

		console.log(`Run command: napi ${args.join(" ")}`);

		const cp = spawn("napi", args, {
			stdio: "inherit",
			shell: true,
			env: envs,
		});

		cp.on("error", reject);
		cp.on("exit", resolve);
	});
}
