const path = require("node:path");
const root = path.resolve(__dirname, "../../");

const setupFilesAfterEnv = [
	"@rspack/test-tools/setup-expect",
	"@rspack/test-tools/setup-env"
];

/** @type {import('jest').Config} */
const wasmConfig = process.env.WASM && {
	setupFilesAfterEnv: [...setupFilesAfterEnv, "@rspack/test-tools/setup-wasm"],
	testPathIgnorePatterns: [
		// Skip because they reply on snapshots
		"Diagnostics.test.js",
		"Error.test.js",
		"StatsAPI.test.js",
		"StatsOutput.test.js",
		// Skip because the loader can not be loaded in CI
		"HotWeb.test.js",
		"HotWorker.test.js",
		"HotNode.test.js",

		// Skip temporarily and should investigate in the future
		"HotSnapshot.hottest.js",
		"Defaults.test.js",
		"Cache.test.js",
		"Compiler.test.js",
		"Serial.test.js",
		"Example.test.js",
		"Incremental-async-node.test.js",
		"Incremental-node.test.js",
		"Incremental-watch-webpack.test.js",
		"Incremental-watch.test.js",
		"Incremental-web.test.js",
		"Incremental-webworker.test.js",
		"NativeWatcher.test.js",
		"NativeWatcher-webpack.test.js"
	],
	maxWorkers: 1,
	maxConcurrency: 1,
	forceExit: true
};

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
	testMatch: process.env.WASM ? [
		"<rootDir>/*.test.js",
	] : [
		"<rootDir>/Serial.test.js",
		"<rootDir>/EsmOutput.test.js",
	],
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
	...(wasmConfig || {}),
	verbose: true,
};

module.exports = config;
