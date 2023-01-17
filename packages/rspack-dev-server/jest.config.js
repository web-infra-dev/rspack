/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
	preset: "ts-jest",
	testEnvironmentOptions: {
		url: "http://localhost/"
	},
	testMatch: ["<rootDir>/tests/*.test.ts", "<rootDir>/tests/e2e/*.test.ts"],
	cache: false,
	testTimeout: process.env.CI ? 60000 : 30000
};
