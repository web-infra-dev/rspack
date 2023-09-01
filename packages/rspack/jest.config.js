process.on("uncaughtException", (err, origin) => {
	console.error(err.message, err.stack, process.exitCode);
});
process.on("unhandledRejection", (err, origin) => {
	console.error(err.message, err.stack, process.exitCode);
});

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
	testTimeout: process.env.CI ? 120000 : 30000,
	testResultsProcessor: "./testProcessor.js",
	verbose: false,
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

module.exports = config;
