require("./a");
require("./b");
require("./c");

it("should has correctly output", () => {
	const fs = require("fs");
	const dir = fs.readdirSync(__dirname);
	expect(dir).toStrictEqual([
		"0.js",
		"3.js", // should be 2.js in webpack, https://github.com/web-infra-dev/rspack/discussions/4697
		"c1.js", // should be c0.js in webpack
		"c2.js",
		"main.js",
		"one.js",
		"two.js"
	]);
});
