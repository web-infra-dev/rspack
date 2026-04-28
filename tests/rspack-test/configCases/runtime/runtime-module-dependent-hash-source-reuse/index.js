const fs = require("node:fs");
const path = require("node:path");

it("should render the final dependent hash source for custom runtime modules", async () => {
	await import(/* webpackChunkName: "async" */ "./async");

	const asyncFilename = fs
		.readdirSync(path.join(__dirname, "chunks"))
		.find(name => name.startsWith("async."));
	expect(asyncFilename).toBeDefined();
	const asyncFilenameMatch = /async\.([0-9a-f]+)\.js/.exec(asyncFilename);
	expect(asyncFilenameMatch).not.toBeNull();
	const [, asyncHash] = asyncFilenameMatch;

	expect(__webpack_require__.dependentHashSourceReuse("async")).toBe(
		`chunks/${asyncFilename}`
	);

	const runtimeSource = fs.readFileSync(
		path.join(__dirname, "runtime.js"),
		"utf-8"
	);
	expect(runtimeSource).toContain(asyncHash);
});
