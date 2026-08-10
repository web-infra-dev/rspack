const fs = require("fs");
const path = require("path");

const content = fs.readFileSync(path.resolve(__dirname, "bundle.js"), "utf-8");

it("should keep wasm new URL untouched in rstest", () => {
	expect(content).toContain('new URL("./test.wasm", import.meta.url)');
});

it("should keep non-wasm new URL behavior", () => {
	expect(content).toContain("/* asset import */");
});
