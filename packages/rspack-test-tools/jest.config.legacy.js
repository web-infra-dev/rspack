const config = require("./jest.config");

/** @type {import('jest').Config} */
module.exports = {
	...config,
	testPathIgnorePatterns: [
		"<rootDir>/tests/Compiler.test.js",
		"<rootDir>/tests/Builtin.test.js",
		"<rootDir>/tests/Defaults.unittest.js",
		"<rootDir>/tests/Stats.test.js",
		"<rootDir>/tests/TreeShaking.test.js",
		"<rootDir>/tests/RuntimeDiff.difftest.js"
	]
};
