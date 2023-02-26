const { spawn } = require("child_process");

let release = process.argv.includes("--release");
build().catch(err => {
	console.error(err);
	process.exit(1);
});

async function build() {
	await new Promise((resolve, reject) => {
		let args = [
			"build",
			"--platform",
			"--dts",
			"binding.d.ts",
			"--js",
			"false"
		];
		if (release) {
			args.push("--release");
		}

		if (process.env.USE_ZIG) {
			args.push("--zig");
		}

		if (process.env.RUST_TARGET) {
			args.push("--target", process.env.RUST_TARGET);
		}

		let cp = spawn("napi", args, {
			stdio: "inherit",
			shell: true
		});

		cp.on("error", reject);
		cp.on("close", resolve);
	});
}
