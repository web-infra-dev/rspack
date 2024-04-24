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
		"<rootDir>/tests/ConfigTestCases.basictest.js",
		"<rootDir>/tests/TestCasesNormal.basictest.js",
		"<rootDir>/tests/HotTestCasesWeb.test.js",
		"<rootDir>/tests/HotTestCasesNode.test.js",
		"<rootDir>/tests/HotTestCasesWebWorker.test.js",
		"<rootDir>/tests/Diagnostics.test.js",
		"<rootDir>/tests/StatsTestCases.basictest.js"
	]
};
