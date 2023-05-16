/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironment: "node",
	testTimeout: process.env.CI ? 120000 : 30000,
	testMatch: ["<rootDir>/tests/**/*.test.ts", "<rootDir>/tests/**/*.test.js"],
	watchPathIgnorePatterns: ["<rootDir>/tests/.*/dist"]
};

if (process.env.CI) {
	config.reporters = [["github-actions", { silent: false }], "summary"];
} else {
	config.reporters = ["default"];
}

module.exports = config;
