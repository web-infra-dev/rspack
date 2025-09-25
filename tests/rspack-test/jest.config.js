const path = require("node:path");
const root = path.resolve(__dirname, "../../");

const setupFilesAfterEnv = [
	"@rspack/test-tools/setup-expect",
	"@rspack/test-tools/setup-env",
	"@rspack/test-tools/setup"
];

/** @type {import('jest').Config} */
const wasmConfig = process.env.WASM && {
	setupFilesAfterEnv: [...setupFilesAfterEnv, "@rspack/test-tools/setup-wasm"],
	testPathIgnorePatterns: [
		// Skip temporarily and should investigate in the future
		"Cache.test.js",
		"Serial.test.js",
		"Normal-hot.test.js",
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
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	setupFilesAfterEnv,
	reporters: [
		["../../scripts/test/ignore-snapshot-default-reporter.cjs", null],
		"../../scripts/test/ignore-snapshot-summary-reporter.cjs"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	testMatch: [
		!process.env.WASM && "<rootDir>/compilerCases/*.js",
		!process.env.WASM && "<rootDir>/errorCases/*.js",
		!process.env.WASM && "<rootDir>/statsAPICases/*.js",
		!process.env.WASM && "<rootDir>/defaultsCases/*/*.js",
		"<rootDir>/*.test.js",
		"<rootDir>/legacy-test/*.test.js"
	].filter(Boolean),
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
	},
	...(wasmConfig || {}),
	verbose: true
};

module.exports = config;
