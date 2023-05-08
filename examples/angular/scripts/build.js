const { spawn, spawnSync } = require("node:child_process");
const ls = spawn("node", ["-v"]);

ls.stdout.on("data", (data) => {
	let res = data.toString().trim();
	const [a, _] = res.split(".");
	if (a >= "v16") {
		build()
			.then(console.log)
			.catch((err) => {
				console.error(err);
				process.exit(-1);
			});
	} else {
		console.log("The Angular CLI requires a minimum of v16.13")
		// Working around angular not support node14, but we need to test node v14 in CI
		process.exit(0);
	}
});

ls.stderr.on("data", (data) => {
	throw new Error(`${data}`);
});

function build() {
	return new Promise((resolve, reject) => {
		try {
			spawnSync("npx", ["rspack", "build"], { stdio: "inherit" });
			resolve();
		} catch (err) {
			reject(err);
		}
	});
}
