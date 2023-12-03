const nodeRequire = require;
const fs = nodeRequire("node:fs/promises");
const path = nodeRequire("node:path");

["a", "b", "c-cjs", "c-mjs"].forEach(fn => {
	it(`[${fn}] should minified as module`, async () => {
		await expect(
			fs.readFile(path.resolve(__dirname, `./${fn}.js`), "utf8")
		).resolves.not.toMatch("__webpack_modules__");
	});
});
