/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	testTimeout: process.env.CI ? 120000 : 30000,
	testMatch: ["<rootDir>/tests/**/*.test.ts", "<rootDir>/tests/**/*.test.js"],
	watchPathIgnorePatterns: ["<rootDir>/tests/.*/dist"]
};

module.exports = config;
