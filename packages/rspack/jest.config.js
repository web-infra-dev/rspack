/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
const config = {
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	testMatch: [
		"<rootDir>/tests/*.test.ts",
		"<rootDir>/tests/*.basictest.ts",
		"<rootDir>/tests/*.longtest.ts",
		"<rootDir>/tests/*.unittest.ts",
		"<rootDir>/tests/copyPlugin/*.test.js",
		"<rootDir>/tests/WatchSuspend.test.js"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	cache: false,
	transform: {
		"^.+\\.(t|j)sx?$": "@swc/jest"
	}
};

module.exports = config;
