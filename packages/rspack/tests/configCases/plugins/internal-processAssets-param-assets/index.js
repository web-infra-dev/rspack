const fs = require("fs");
const path = require("path");

it("delete assets[filename] should works", () => {
	expect(fs.existsSync(path.resolve(__dirname, "dup.txt"))).toBe(false);
	const keys = fs.readFileSync(
		path.resolve(__dirname, "assets-keys.txt"),
		"utf-8"
	);
	expect(keys).toBe("main.js,main.js.map,dup.txt\nmain.js,main.js.map");
});
