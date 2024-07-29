/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironmentOptions: {
		url: "http://localhost/"
	},
	testMatch: ["<rootDir>/tests/*.test.ts", "<rootDir>/tests/e2e/*.test.ts"],
	cache: false,
	testTimeout: process.env.CI ? 120000 : 30000,
	transform: {
		"(.*)\\.{js,ts}": [
			"ts-jest",
			{
				tsconfig: "<rootDir>/tests/tsconfig.json"
			}
		]
	}
};

module.exports = config;
