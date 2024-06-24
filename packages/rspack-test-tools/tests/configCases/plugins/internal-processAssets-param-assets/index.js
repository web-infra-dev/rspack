const fs = require("fs");
const path = require("path");

it("delete assets[filename] should works", () => {
	expect(fs.existsSync(path.resolve(__dirname, "dup.txt"))).toBe(false);
	const keys = fs.readFileSync(
		path.resolve(__dirname, "assets-keys.txt"),
		"utf-8"
	);
	expect(keys).toBe(
		"bundle0.js.map,bundle0.js,dup.txt\nbundle0.js.map,bundle0.js"
	);
});
