const fs = require("fs");
const path = require("path");

it("should keep function name", () => {
	let a = fs.readFileSync(path.resolve(__dirname, "./a.js"), "utf-8");
	expect(a).toContain("fname");
});

it("should keep ident name", () => {
	let a = fs.readFileSync(path.resolve(__dirname, "./a.js"), "utf-8");
	expect(a).toContain(
		globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
			? "__rspack_modules"
			: "__webpack_modules__"
	);
});
