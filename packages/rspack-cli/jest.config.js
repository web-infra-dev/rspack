/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const wasmConfig = process.env.WASM && {
	testPathIgnorePatterns: [
		"profile.test.ts" // Skip due to lack of system api support
	],
	maxWorkers: 1,
	maxConcurrency: 1
};

/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	testTimeout: process.env.CI ? 200000 : 30000,
	testMatch: ["<rootDir>/tests/**/*.test.ts", "<rootDir>/tests/**/*.test.js"],
	watchPathIgnorePatterns: ["<rootDir>/tests/.*/dist"],
	extensionsToTreatAsEsm: [".mts"],
	transform: {
		"^.+\\.(ts)?$": ["ts-jest", { tsconfig: "<rootDir>/tests/tsconfig.json" }]
	},
	cache: false,
	prettierPath: require.resolve("prettier-2"),
	...(wasmConfig || {})
};

module.exports = config;
