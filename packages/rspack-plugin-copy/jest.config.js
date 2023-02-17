module.exports = {
	testEnvironment: "node",
	testMatch: ["<rootDir>/tests/*.test.js"],
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
