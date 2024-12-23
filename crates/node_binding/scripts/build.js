const path = require("path");
const { readFileSync, writeFileSync } = require("fs")

const { spawn } = require("child_process");

const CARGO_SAFELY_EXIT_CODE = 0;

// Faster release for CI & canary with `thin` LTO
let release = process.argv.includes("--release");
// Slower release for production with `fat` LTO
let releaseProd = process.argv.includes("--release-prod");
let releaseDebug = process.argv.includes("--release-debug");
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
		if (release) {
			args.push("--release");
		}
		if (releaseProd) {
			args.push('--profile release-prod');
		}
		if (releaseDebug) {
			args.push('--profile release-debug');
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

		console.log(`Run command: napi ${args.join(' ')}`);

		let cp = spawn("napi", args, {
			stdio: "inherit",
			shell: true
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
