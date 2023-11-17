require("./a");
require("./b");
require("./c");

it("should has correctly output", () => {
	const fs = require("fs");
	const dir = fs.readdirSync(__dirname);
	expect(dir).toStrictEqual([
		"0.js",
		"2.js",
		"c0.js",
		"c2.js",
		"main.js",
		"one.js",
		"two.js"
	]);
});
