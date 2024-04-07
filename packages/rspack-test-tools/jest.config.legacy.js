const config = require("./jest.config");

/** @type {import('jest').Config} */
module.exports = {
	...config,
	// can only use filename otherwise will fail by snapshot obsolete
	testPathIgnorePatterns: [
		"Compiler.test.js",
		"Defaults.unittest.js",
		"Stats.test.js",
		"TreeShaking.test.js",
		"Builtin.test.js",
		".difftest.js"
	]
};
