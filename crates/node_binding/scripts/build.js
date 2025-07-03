const { values, positionals } = require("util").parseArgs({
	args: process.argv.slice(2),
	options: {
		profile: {
			type: "string"
		}
	},
	strict: true,
	allowPositionals: true
});

const { spawn } = require("child_process");

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
		let args = [
			"--dts",
			"binding.d.ts",
			"--no-dts-cache",
		];
		let features = [];
		let envs = { ...process.env };

		if (process.env.RUST_TARGET) {
			args.push("--target", process.env.RUST_TARGET);
		}
		if (!process.env.DISABLE_PLUGIN) {
			args.push("--no-default-features");
			features.push("plugin");
		}
		if (values.profile === "release-debug" &&
			(!process.env.RUST_TARGET || process.env.RUST_TARGET.includes("linux") || process.env.RUST_TARGET.includes("darwin"))
		) {
			features.push("sftrace-setup");
			envs.RUSTFLAGS = "-Zinstrument-xray=always";
		}

		if (features.length) {
			args.push("--features", features.join(","));
		}

		if (positionals.length > 0) {
			args.push(...positionals);
		}

		let cp = spawn("rspack-builder", args, {
			stdio: "inherit",
			shell: true,
			env: envs,
		});
		cp.on("error", reject);
		cp.on("exit", resolve);
	});
}
