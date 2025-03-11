const path = require("node:path");

const root = path.resolve(__dirname, "../../");
/** @type {import('jest').Config} */
const config = {
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	setupFilesAfterEnv: [
		"@rspack/test-tools/setup-expect",
		"@rspack/test-tools/setup-env"
	],
	reporters: [
		["../../scripts/test/ignore-snapshot-default-reporter.cjs", null],
		"../../scripts/test/ignore-snapshot-summary-reporter.cjs"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	testMatch: [
		"<rootDir>/tests/*.test.js",
		"<rootDir>/tests/legacy-test/*.test.js"
	],
	moduleNameMapper: {
		// Fixed jest-serialize-path not working when non-ascii code contains.
		slash: path.join(__dirname, "../../scripts/test/slash.cjs"),
		// disable sourcmap remapping for ts file
		"source-map-support/register": "identity-obj-proxy"
	},
	cache: !process.env.CI,
	transformIgnorePatterns: [root],
	maxWorkers: "80%",
	snapshotFormat: {
		escapeString: true,
		printBasicPrototype: true
	},
	globals: {
		updateSnapshot:
			process.argv.includes("-u") || process.argv.includes("--updateSnapshot"),
		testFilter:
			process.argv.includes("--test") || process.argv.includes("-t")
				? process.argv[
						(process.argv.includes("-t")
							? process.argv.indexOf("-t")
							: process.argv.indexOf("--test")) + 1
					]
				: undefined,
		printLogger: process.argv.includes("--verbose")
	}
};

module.exports = config;
