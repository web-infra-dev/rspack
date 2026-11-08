const fs = require("fs");
const path = require("path");

it("should render import.source() for externalized module instead of import()", () => {
	const output = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	expect(output).toContain('import.source("./add.wasm")');
	expect(output).not.toContain('import("./add.wasm")');
});
