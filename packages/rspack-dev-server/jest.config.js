/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironmentOptions: {
		url: "http://localhost/"
	},
	testMatch: ["<rootDir>/tests/*.test.ts", "<rootDir>/tests/e2e/*.test.ts"],
	cache: false,
	testTimeout: process.env.CI ? 120000 : 30000
};

if (process.env.CI) {
	config.reporters = [["github-actions", { silent: false }], "summary"];
} else {
	config.reporters = ["default"];
}

module.exports = config;
