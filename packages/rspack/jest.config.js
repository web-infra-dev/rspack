/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
const config = {
	testEnvironment: "node",
	testMatch: [
		"<rootDir>/tests/*.test.ts",
		"<rootDir>/tests/*.basictest.ts",
		"<rootDir>/tests/*.longtest.ts",
		"<rootDir>/tests/*.unittest.ts",
		"<rootDir>/tests/copyPlugin/*.test.js",
		"<rootDir>/tests/WatchSuspend.test.js"
	],
	testTimeout: 120000,
	cache: false,
	transform: {
		"^.+\\.tsx?$": [
			"ts-jest",
			{
				isolatedModules: true
			}
		],
		"^.+\\.jsx?$": "babel-jest"
	}
};

if (process.env.CI) {
	config.reporters = [["github-actions", { silent: false }], "summary"];
} else {
	config.reporters = ["default"];
}

module.exports = config;
