const path = require("path");

/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const config = {
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	setupFilesAfterEnv: ["<rootDir>/tests/setupTestFramework.js"],
	testMatch: [
		"<rootDir>/tests/*.test.ts",
		"<rootDir>/tests/*.test.js",
		"<rootDir>/tests/*.basictest.ts",
		"<rootDir>/tests/*.basictest.js",
		"<rootDir>/tests/*.longtest.ts",
		"<rootDir>/tests/*.longtest.js",
		"<rootDir>/tests/*.unittest.ts",
		"<rootDir>/tests/*.unittest.js",
		"<rootDir>/tests/copyPlugin/*.test.js",
		"<rootDir>/tests/WatchSuspend.test.js"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	moduleNameMapper: {
		// Fixed jest-serialize-path not working when non-ascii code contains.
		slash: path.join(__dirname, "../../scripts/test/slash.cjs")
	}
};

module.exports = config;
