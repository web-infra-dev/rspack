const path = require("path");

const root = path.resolve(__dirname, "../../");
/** @type {import('jest').Config} */
const config = {
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	setupFilesAfterEnv: ["../rspack/tests/setupTestFramework.js"],
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	testMatch: [
		"<rootDir>/tests/*.test.js",
		"<rootDir>/tests/*.basictest.js",
		"<rootDir>/tests/*.longtest.js",
		"<rootDir>/tests/*.unittest.js"
	],
	moduleNameMapper: {
		// Fixed jest-serialize-path not working when non-ascii code contains.
		slash: path.join(__dirname, "../../scripts/test/slash.cjs"),
		// disable sourcmap remapping for ts file
		"source-map-support/register": "identity-obj-proxy"
	},
	cache: false,
	transformIgnorePatterns: [root],
	snapshotFormat: {
		escapeString: true,
		printBasicPrototype: true
	},
	globals: {
		updateSnapshot:
			process.argv.includes("-u") || process.argv.includes("--updateSnapshot")
	},
	snapshotResolver:
		process.env.SNAPSHOT === "legacy"
			? path.join(__dirname, "./snapshot-resolver.js")
			: undefined
};

module.exports = config;
