const { spawn } = require("child_process");

const CARGO_SAFELY_EXIT_CODE = 0;

let release = process.argv.includes("--release");
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
			"--no-const-enum"
		];
		if (release) {
			args.push("--release");
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

		let cp = spawn("napi", args, {
			stdio: "inherit",
			shell: true
		});

		cp.on("error", reject);
		cp.on("close", resolve);
	});
}
