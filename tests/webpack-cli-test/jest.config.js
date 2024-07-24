module.exports = {
	testEnvironment: "node",
	collectCoverage: true,
	coverageDirectory: ".nyc_output",
	coverageReporters: ["json"],
	coveragePathIgnorePatterns: ["<rootDir>/test/"],
	transform: {
		"^.+\\.(ts)?$": "ts-jest"
	},
	testRegex: ["/.*\\.(test.js|test.ts)$"],
	moduleFileExtensions: ["ts", "js", "json"],
	snapshotResolver: "<rootDir>/scripts/snapshot-resolver.js",
	watchPlugins: [
		"jest-watch-typeahead/filename",
		"jest-watch-typeahead/testname"
	],
	setupFilesAfterEnv: ["<rootDir>/scripts/setup-test.js"],
	globalTeardown: "<rootDir>/scripts/cleanup-test.js",
	globalSetup: "<rootDir>/scripts/global-setup.js",
	modulePathIgnorePatterns: [
		"<rootDir>/test/loader/test-loader",
		"<rootDir>/test/plugin/test-plugin"
	]
};
