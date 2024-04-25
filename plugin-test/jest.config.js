const path = require("path");


const root = path.resolve(__dirname, "../");

/** @type {import('jest').Config} */
const config = {
	testEnvironment: "../scripts/test/patch-node-env.cjs",
	testMatch: [
		"<rootDir>/**/*.test.js"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	cache: false,
	transformIgnorePatterns: [root],
	setupFilesAfterEnv: ["<rootDir>/setupTestEnv.js"],
	snapshotFormat: {
		escapeString: true,
		printBasicPrototype: true
	}
};

module.exports = config;
