const config = require("./jest.config");

/** @type {import('jest').Config} */
module.exports = {
	...config,
	testMatch: [
		"<rootDir>/tests/Compiler.test.js",
		"<rootDir>/tests/Builtin.test.js",
		"<rootDir>/tests/Defaults.unittest.js",
		"<rootDir>/tests/Stats.unittest.js",
		"<rootDir>/tests/TreeShaking.test.js",
		"<rootDir>/tests/ConfigTestCases.basictest.js"
	]
};
