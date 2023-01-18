const fs = require("fs");
const path = require("path");

it("delete assets[filename] should works", () => {
	expect(fs.existsSync(path.resolve(__dirname, "main.js"))).toBe(false);
});
