process.on("uncaughtException", (err, origin) => {
	console.error(err.message, err.stack, process.exitCode);
});
process.on("unhandledRejection", (err, origin) => {
	console.error(err.message, err.stack, process.exitCode);
});

["log", "warn", "error"].forEach(methodName => {
	const originalMethod = console[methodName];
	console[methodName] = (...args) => {
		let initiator = "unknown place";
		try {
			throw new Error();
		} catch (e) {
			if (typeof e.stack === "string") {
				let isFirst = true;
				for (const line of e.stack.split("\n")) {
					const matches = line.match(/^\s+at\s+(.*)/);
					if (matches) {
						if (!isFirst) {
							// first line - current function
							// second line - caller (what we are looking for)
							initiator = matches[1];
							break;
						}
						isFirst = false;
					}
				}
			}
		}
		originalMethod.apply(console, [...args, "\n", `  at ${initiator}`]);
	};
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
