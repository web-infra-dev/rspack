/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
	preset: "ts-jest",
	testEnvironment: "node",
	testTimeout: process.env.CI ? 120000 : 30000,
	watchPathIgnorePatterns: ["<rootDir>/tests/.*/dist"]
};
