const path = require("path");
const { readFileSync, writeFileSync } = require("fs")
const { values } = require('util').parseArgs({
	args: process.argv.slice(2),
	options: {
		profile: {
			type: 'string'
		}
	}
})

const { spawn } = require("child_process");

const CARGO_SAFELY_EXIT_CODE = 0;

let watch = process.argv.includes("--watch");

build().then((value) => {
	// Regarding cargo's non-zero exit code as an error.
	if (value !== CARGO_SAFELY_EXIT_CODE) {
		process.exit(value)
	}
}).catch(err => {
	console.error(err);
	process.exit(1);
})

async function build() {
	return new Promise((resolve, reject) => {
		let args = [
			"build",
			"--platform",
			"--dts",
			"binding.d.ts",
			"--no-js",
			// "--no-const-enum",
			"--no-dts-header",
			"--pipe",
			`"node ./scripts/dts-header.js"`
		];
		if (values.profile) {
			args.push("--profile", values.profile)
		}
		if (watch) {
			args.push("--watch");
		}
		if (process.env.USE_ZIG) {
			args.push("--cross-compile");
		}
		if (process.env.RUST_TARGET) {
			args.push("--target", process.env.RUST_TARGET);
		}
		if (!process.env.DISABLE_PLUGIN) {
			args.push("--no-default-features");
			args.push("--features plugin");
		}
		args.push("--no-dts-cache");

		console.log(`Run command: napi ${args.join(' ')}`);

		let cp = spawn("napi", args, {
			stdio: "inherit",
			shell: true,
		});

		cp.on("error", reject);
		cp.on("close", () => {
			// Fix an issue where napi cli does not generate `string_enum` with `enum`s.
			let dts = path.resolve(__dirname, "../binding.d.ts");
			writeFileSync(dts, readFileSync(dts, "utf8").replaceAll("const enum", "enum"));

			resolve(null)
		});
	});
}
