const path = require("path");
const root = path.resolve(__dirname, "../");

/** @type {import('jest').Config} */
const config = {
	preset: "ts-jest",
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	testMatch: [
		"<rootDir>/**/*.test.js",
		"<rootDir>/**/*.test.ts"
	],
	transform: {
		"^.+\\.(ts)?$": ["ts-jest", { tsconfig: "<rootDir>/tsconfig.json" }]
	},
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	cache: false,
	setupFilesAfterEnv: ["<rootDir>/setupTestEnv.js"],
	snapshotFormat: {
		escapeString: true,
		printBasicPrototype: true
	},
	globals: {
		updateSnapshot:
			process.argv.includes("-u") || process.argv.includes("--updateSnapshot")
	},
	extensionsToTreatAsEsm: [".mts"]
};

module.exports = config;
