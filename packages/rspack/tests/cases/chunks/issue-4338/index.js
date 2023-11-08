require("./a");
require("./b");
require("./c");

it("should has correctly output", () => {
	const fs = require("fs");
	const dir = fs.readdirSync(__dirname);
	expect(dir).toStrictEqual([
		"2.js",
		"4.js",
		"c2.js",
		"c4.js",
		"main.js",
		"one.js",
		"two.js"
	]);
});
