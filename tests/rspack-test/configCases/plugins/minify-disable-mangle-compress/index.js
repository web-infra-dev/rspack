const fs = require("fs");
const path = require("path");

it("should keep function name", () => {
	let a = fs.readFileSync(path.resolve(__dirname, "./a.js"), "utf-8");
	expect(a).toContain("fname");
});

it("should keep ident name", () => {
	let a = fs.readFileSync(path.resolve(__dirname, "./a.js"), "utf-8");
	expect(a).toContain("__webpack_modules__");
});
