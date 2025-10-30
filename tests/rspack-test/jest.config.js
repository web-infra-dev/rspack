const path = require("node:path");
const root = path.resolve(__dirname, "../../");

const setupFilesAfterEnv = [
	"@rspack/test-tools/setup-expect",
	"@rspack/test-tools/setup-env"
];

/** @type {import('jest').Config} */
const config = {
	testEnvironment: "@rspack/test-tools/jest/patch-node-env",
	setupFilesAfterEnv,
	reporters: [
		["@rspack/test-tools/jest/ignore-snapshot-default-reporter", null],
		"@rspack/test-tools/jest/ignore-snapshot-summary-reporter"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	testMatch: [],
	testPathIgnorePatterns: ["<rootDir>"],
	moduleNameMapper: {
		// Fixed jest-serialize-path not working when non-ascii code contains.
		slash: "@rspack/test-tools/jest/slash",
		// disable sourcemap remapping for ts file
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
		printLogger: process.argv.includes("--verbose"),
		__TEST_PATH__: __dirname,
		__TEST_FIXTURES_PATH__: path.resolve(__dirname, "fixtures"),
		__TEST_DIST_PATH__: path.resolve(__dirname, "js"),
		__ROOT_PATH__: root,
		__RSPACK_PATH__: path.resolve(root, "packages/rspack"),
		__RSPACK_TEST_TOOLS_PATH__: path.resolve(root, "packages/rspack-test-tools"),
		__DEBUG__: process.env.DEBUG === "test",
	},
	verbose: true,
};

module.exports = process.env.WASM ? { testPathIgnorePatterns: [".*"], passWithNoTests: true } : config;
