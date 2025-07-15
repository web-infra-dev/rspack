const fs = require("fs");
const path = require("path");

it("should keep function name", () => {
	const a = fs.readFileSync(path.resolve(__dirname, "./a.js"), "utf-8");
	expect(a).toContain("fname");
});

it("should keep ident name", () => {
	const a = fs.readFileSync(path.resolve(__dirname, "./a.js"), "utf-8");
	expect(a).toContain("__webpack_modules__");
});
