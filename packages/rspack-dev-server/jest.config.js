const os = require("node:os");
const isWin = os.platform() === "win32";
/** @type {import('ts-jest/dist/types').JestConfigWithTsJest} */
const config = {
	preset: "ts-jest",
	testEnvironmentOptions: {
		url: "http://localhost/"
	},
	testMatch: [
		"<rootDir>/tests/*.test.ts",
		...(isWin ? [] : ["<rootDir>/tests/e2e/*.test.js"])
	],
	testPathIgnorePatterns: isWin
		? []
		: [
				// TODO: check why http proxy server throw error with websocket server
				"<rootDir>/tests/e2e/allowed-hosts.test.js",
				// TODO: check why this test timeout
				"<rootDir>/tests/e2e/host.test.js",
				// TODO: check why this test throw error when run with other tests
				"<rootDir>/tests/e2e/watch-files.test.js",
				// TODO: check why this test timeout
				"<rootDir>/tests/e2e/web-socket-server-url.test.js"
			],
	cache: false,
	testTimeout: process.env.CI ? 120000 : 30000,
	transform: {
		"(.*)\\.{js,ts}": [
			"ts-jest",
			{
				tsconfig: "<rootDir>/tests/tsconfig.json"
			}
		]
	},
	// Add this to find out which test timeouts
	// testSequencer: "<rootDir>/tests/helpers/sequencer.js",
	snapshotResolver: "<rootDir>/tests/helpers/snapshot-resolver.js",
	setupFilesAfterEnv: ["<rootDir>/tests/helpers/setup-test.js"],
	globalSetup: "<rootDir>/tests/helpers/global-setup-test.js",
	moduleNameMapper: {
		"^uuid$": require.resolve("uuid")
	}
};

module.exports = config;
