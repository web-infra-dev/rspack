const fs = require("node:fs");
const path = require("node:path");

it("should render the final built-in get script filename runtime source", async () => {
	await import(/* webpackChunkName: "async" */ "./async");

	const asyncFilename = fs
		.readdirSync(path.join(__dirname, "chunks"))
		.find(name => name.startsWith("async."));
	const [, asyncHash] = /async\.([0-9a-f]+)\.js/.exec(asyncFilename);

	expect(__webpack_get_script_filename__("async")).toBe(
		`chunks/${asyncFilename}`
	);

	const runtimeSource = fs.readFileSync(
		path.join(__dirname, "runtime.js"),
		"utf-8"
	);
	expect(runtimeSource).toContain(asyncHash);
});
