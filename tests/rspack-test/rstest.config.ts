import path from 'node:path';
import { defineConfig } from '@rstest/core';
const root = path.resolve(__dirname, "../../");

const setupFilesAfterEnv = [
	"@rspack/test-tools/setup-env",
	"@rspack/test-tools/setup-expect",
];

export default defineConfig({
	setupFiles: setupFilesAfterEnv,
	testTimeout: process.env.CI ? 60000 : 30000,
	include: process.env.WASM ? [] : [
		"<rootDir>/*.test.js",
	],
	exclude: ["Cache.test.js", "Incremental-*.test.js", "Hot*.test.js", "Serial.test.js", "NativeWatcher*.test.js", "Diagnostics.test.js", "EsmOutput.test.js"],
	slowTestThreshold: 5000,
	resolve: {
		alias: {
			// Fixed jest-serialize-path not working when non-ascii code contains.
			slash: path.join(__dirname, "../../scripts/test/slash.cjs"),
			// disable sourcemap remapping for ts file
			"source-map-support/register": "identity-obj-proxy"
		}
	},
	source: {
		exclude: [root],
	},
	globals: true,
	output: {
		externals: [/.*/],
	},
	passWithNoTests: true,
	snapshotFormat: {
		escapeString: true,
		printBasicPrototype: true
	},
	pool: {
		maxWorkers: "80%",
		execArgv: ['--no-warnings', '--expose-gc', '--max-old-space-size=8192', '--experimental-vm-modules'],
	},
	env: {
		updateSnapshot:
			process.argv.includes("-u") || process.argv.includes("--updateSnapshot") ? 'true' : 'false',
		RSPACK_DEV: 'false',
		RSPACK_EXPERIMENTAL: 'true',
		RSPACK_CONFIG_VALIDATE: "strict",
		testFilter:
			process.argv.includes("--test") || process.argv.includes("-t")
				? process.argv[
				(process.argv.includes("-t")
					? process.argv.indexOf("-t")
					: process.argv.indexOf("--test")) + 1
				]
				: undefined,
		printLogger: process.argv.includes("--verbose") ? 'true' : 'false',
		__TEST_PATH__: __dirname,
		__TEST_FIXTURES_PATH__: path.resolve(__dirname, "fixtures"),
		__TEST_DIST_PATH__: path.resolve(__dirname, "js"),
		__ROOT_PATH__: root,
		__RSPACK_PATH__: path.resolve(root, "packages/rspack"),
		__RSPACK_TEST_TOOLS_PATH__: path.resolve(root, "packages/rspack-test-tools"),
	},
	reporters: process.env.CI ? undefined : ["verbose"],
	hideSkippedTests: true,
});

