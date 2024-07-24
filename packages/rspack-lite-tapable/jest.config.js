/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	testTimeout: process.env.CI ? 200000 : 30000,
	testMatch: ["<rootDir>/tests/**/*.test.ts", "<rootDir>/tests/**/*.test.js"],
	watchPathIgnorePatterns: ["<rootDir>/tests/.*/dist"],
	extensionsToTreatAsEsm: [".mts"]
};

module.exports = config;
