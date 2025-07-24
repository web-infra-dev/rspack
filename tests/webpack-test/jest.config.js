const root = require("path").resolve(__dirname, "../");

const wasmConfig = process.env.WASM && {
	testPathIgnorePatterns: [
		// Skip because they reply on snapshots
		"<rootDir>/StatsTestCases.basictest.js",
		// Skip temporarily and should investigate in the future
		"<rootDir>/ConfigTestCases.basictest.js",
		"<rootDir>/HotTestCasesWeb.test.js",
		"<rootDir>/Watch.test.js",
	],
	maxWorkers: 1,
	maxConcurrency: 1,
	forceExit: true
};

module.exports = {
	"forceExit": false,
	"setupFiles": [
		"<rootDir>/setupEnv.js"
	],
	"setupFilesAfterEnv": [
		"<rootDir>/setupTestFramework.js"
	],
	"testMatch": [
		"<rootDir>/HotTestCasesWeb.test.js",
	],
	"watchPathIgnorePatterns": [
		"<rootDir>/.git",
		"<rootDir>/node_modules",
		"<rootDir>/js",
		"<rootDir>/browsertest/js",
		"<rootDir>/fixtures/temp-cache-fixture",
		"<rootDir>/fixtures/temp-",
		"<rootDir>/benchmark",
		"<rootDir>/assembly",
		"<rootDir>/tooling",
		"<rootDir>/examples/*/dist",
		"<rootDir>/coverage",
		"<rootDir>/.eslintcache"
	],
	"modulePathIgnorePatterns": [
		"<rootDir>/.git",
		"<rootDir>/node_modules/webpack/node_modules",
		"<rootDir>/js",
		"<rootDir>/browsertest/js",
		"<rootDir>/fixtures/temp-cache-fixture",
		"<rootDir>/fixtures/temp-",
		"<rootDir>/benchmark",
		"<rootDir>/examples/*/dist",
		"<rootDir>/coverage",
		"<rootDir>/.eslintcache"
	],
	"transformIgnorePatterns": [
		root
	],
	"coverageDirectory": "<rootDir>/coverage",
	"coveragePathIgnorePatterns": [
		"\\.runtime\\.js$",
		"<rootDir>",
		"<rootDir>/schemas",
		"<rootDir>/node_modules"
	],
	"testEnvironment": "./patch-node-env.js",
	"prettierPath": require.resolve("prettier-2"),
	"snapshotFormat": {
		"escapeString": true,
		"printBasicPrototype": true
	},
	"reporters": [
		["../../scripts/test/ignore-snapshot-default-reporter.cjs", null],
		"../../scripts/test/ignore-snapshot-summary-reporter.cjs"
	],
	...(wasmConfig || {})
}
