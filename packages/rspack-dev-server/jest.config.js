/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironmentOptions: {
		url: "http://localhost/"
	},
	testMatch: ["<rootDir>/tests/*.test.ts", "<rootDir>/tests/e2e/*.test.js"],
	cache: false,
	testTimeout: process.env.CI ? 120000 : 30000,
	transform: {
		"(.*)\\.{js,ts}": [
			"ts-jest",
			{
				tsconfig: "<rootDir>/tests/tsconfig.json"
			}
		]
	},
	snapshotResolver: "<rootDir>/tests/helpers/snapshot-resolver.js",
	setupFilesAfterEnv: ["<rootDir>/tests/helpers/setup-test.js"],
	globalSetup: "<rootDir>/tests/helpers/global-setup-test.js",
	moduleNameMapper: {
		"^uuid$": require.resolve("uuid")
	}
};

module.exports = config;
