const { spawn, spawnSync } = require("node:child_process");
const path = require("node:path");
const ls = spawn("node", ["-v"]);

ls.stdout.on("data", data => {
	let res = data.toString().trim();
	const [a, _] = res.split(".");
	if (a >= "v16") {
		build()
			.then(console.log)
			.catch(err => {
				console.error(err);
				process.exit(1);
			});
	} else {
		console.log("The Angular CLI requires a minimum of v16.13");
		// Working around angular not support node14, but we need to test node v14 in CI
		process.exit(0);
	}
});

ls.stderr.on("data", data => {
	throw new Error(`${data}`);
});

function build() {
	return new Promise((resolve, reject) => {
		try {
			const { status, error, stderr } = spawnSync(
				"node",
				[
					path.resolve(__dirname, "../node_modules/@rspack/cli/bin/rspack"),
					"build"
				],
				{
					stdio: "inherit"
				}
			);
			if (status !== 0) {
				reject(stderr?.toString() || error);
				return;
			}
			resolve();
		} catch (err) {
			reject(err);
		}
	});
}
