const path = require("path");

/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const config = {
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	testMatch: [
		"<rootDir>/tests/*.test.ts",
		"<rootDir>/tests/*.test.js",
		"<rootDir>/tests/*.basictest.ts",
		"<rootDir>/tests/*.basictest.js",
		"<rootDir>/tests/*.longtest.ts",
		"<rootDir>/tests/*.longtest.js",
		"<rootDir>/tests/*.unittest.ts",
		"<rootDir>/tests/copyPlugin/*.test.js",
		"<rootDir>/tests/cssExtract/*.test.js",
		"<rootDir>/tests/WatchSuspend.test.js"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	cache: false,
	transform: {
		"^.+\\.(t|j)sx?$": "@swc/jest"
	},
	globals: {
		"ts-jest": {
			tsconfig: "<rootDir>/tests/tsconfig.json"
		}
	},
	moduleNameMapper: {
		// Fixed jest-serialize-path not working when non-ascii code contains.
		slash: path.join(__dirname, "../../scripts/test/slash.cjs")
	}
};

module.exports = config;
