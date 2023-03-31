/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
	preset: "ts-jest",
	testEnvironment: "node",
	testMatch: ["**/fixtures/**/test/*.test.ts"],
	globalSetup: "./scripts/globalSetup.ts",
	globalTeardown: "./scripts/globalTearDown.ts",
	testEnvironment: "./scripts/env.ts",
	setupFilesAfterEnv: ["./scripts/setupFiles.ts"],
	testTimeout: process.env.CI ? 60 * 1000 : 30 * 1000,
	cache: false
};
