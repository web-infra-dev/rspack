/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
	testEnvironment: "node",
	testMatch: [
		"<rootDir>/tests/*.test.ts",
		"<rootDir>/tests/*.basictest.ts",
		"<rootDir>/tests/*.longtest.ts",
		"<rootDir>/tests/*.unittest.ts",
		"<rootDir>/tests/copyPlugin/*.test.js"
	],
	testTimeout: 30000,
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
