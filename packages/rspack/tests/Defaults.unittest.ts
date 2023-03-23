// @ts-nocheck
const path = require("path");
const jestDiff = require("jest-diff").diff;
const stripAnsi = require("strip-ansi");
import { getNormalizedRspackOptions, applyRspackOptionsDefaults } from "../src";
/**
 * Escapes regular expression metacharacters
 * @param {string} str String to quote
 * @returns {string} Escaped string
 */
const quoteMeta = str => {
	return str.replace(/[-[\]\\/{}()*+?.^$|]/g, "\\$&");
};

const cwd = process.cwd();
const cwdRegExp = new RegExp(
	`${quoteMeta(cwd)}((?:\\\\)?(?:[a-zA-Z.\\-_]+\\\\)*)`,
	"g"
);
const escapedCwd = JSON.stringify(cwd).slice(1, -1);
const escapedCwdRegExp = new RegExp(
	`${quoteMeta(escapedCwd)}((?:\\\\\\\\)?(?:[a-zA-Z.\\-_]+\\\\\\\\)*)`,
	"g"
);
const normalize = str => {
	if (cwd.startsWith("/")) {
		str = str.replace(new RegExp(quoteMeta(cwd), "g"), "<cwd>");
	} else {
		str = str.replace(cwdRegExp, (m, g) => `<cwd>${g.replace(/\\/g, "/")}`);
		str = str.replace(
			escapedCwdRegExp,
			(m, g) => `<cwd>${g.replace(/\\\\/g, "/")}`
		);
	}
	str = str.replace(/@@ -\d+,\d+ \+\d+,\d+ @@/g, "@@ ... @@");
	return str;
};

class Diff {
	value: any;
	constructor(value) {
		this.value = value;
	}
}

expect.addSnapshotSerializer({
	test(value) {
		return value instanceof Diff;
	},
	print(received: any) {
		return normalize(received.value);
	}
});

expect.addSnapshotSerializer({
	test(value) {
		return typeof value === "string";
	},
	print(received) {
		return JSON.stringify(normalize(received));
	}
});

const getDefaultConfig = config => {
	config = getNormalizedRspackOptions(config);
	applyRspackOptionsDefaults(config);
	process.chdir(cwd);
	return config;
};

describe("snapshots", () => {
	const baseConfig = getDefaultConfig({ mode: "none" });

	it("should have the correct base config", () => {
		expect(baseConfig).toMatchInlineSnapshot(`
		{
		  "builtins": {
		    "copy": undefined,
		    "css": {
		      "modules": {
		        "exportsOnly": false,
		        "localIdentName": "[path][name][ext]__[local]",
		        "localsConvention": "asIs",
		      },
		    },
		    "decorator": {
		      "emitMetadata": true,
		      "legacy": true,
		    },
		    "define": {},
		    "devFriendlySplitChunks": false,
		    "emotion": undefined,
		    "html": [],
		    "minifyOptions": undefined,
		    "noEmitAssets": false,
		    "pluginImport": undefined,
		    "postcss": {
		      "pxtorem": undefined,
		    },
		    "presetEnv": undefined,
		    "progress": undefined,
		    "react": {},
		    "relay": undefined,
		    "treeShaking": false,
		  },
		  "cache": false,
		  "context": "<cwd>",
		  "dependencies": undefined,
		  "devServer": undefined,
		  "devtool": false,
		  "entry": {
		    "main": {
		      "import": [
		        "./src",
		      ],
		    },
		  },
		  "experiments": {
		    "incrementalRebuild": true,
		    "lazyCompilation": false,
		  },
		  "externals": undefined,
		  "externalsPresets": {
		    "node": false,
		  },
		  "externalsType": "var",
		  "infrastructureLogging": {},
		  "mode": "none",
		  "module": {
		    "defaultRules": [
		      {
		        "test": /\\\\\\.json\\$/i,
		        "type": "json",
		      },
		      {
		        "test": /\\\\\\.mjs\\$/i,
		        "type": "javascript/esm",
		      },
		      {
		        "test": /\\\\\\.cjs\\$/i,
		        "type": "javascript/dynamic",
		      },
		      {
		        "test": /\\\\\\.jsx\\$/i,
		        "type": "jsx",
		      },
		      {
		        "test": /\\\\\\.ts\\$/i,
		        "type": "ts",
		      },
		      {
		        "test": /\\\\\\.tsx\\$/i,
		        "type": "tsx",
		      },
		      {
		        "oneOf": [
		          {
		            "test": /\\\\\\.module\\\\\\.css\\$/i,
		            "type": "css/module",
		          },
		          {
		            "resolve": {
		              "preferRelative": true,
		            },
		            "type": "css",
		          },
		        ],
		        "test": /\\\\\\.css\\$/i,
		      },
		    ],
		    "parser": {
		      "asset": {
		        "dataUrlCondition": {
		          "maxSize": 8096,
		        },
		      },
		    },
		    "rules": [],
		  },
		  "name": undefined,
		  "node": {
		    "__dirname": "warn-mock",
		    "__filename": "warn-mock",
		    "global": "warn",
		  },
		  "optimization": {
		    "minimize": false,
		    "minimizer": [],
		    "moduleIds": "named",
		    "removeAvailableModules": true,
		    "runtimeChunk": false,
		    "sideEffects": "flag",
		    "splitChunks": {
		      "cacheGroups": {
		        "default": {
		          "idHint": "",
		          "minChunks": 2,
		          "priority": -20,
		          "reuseExistingChunk": true,
		        },
		        "defaultVendors": {
		          "idHint": "vendors",
		          "priority": -10,
		          "reuseExistingChunk": true,
		          "test": /\\[\\\\\\\\/\\]node_modules\\[\\\\\\\\/\\]/i,
		        },
		      },
		      "chunks": "async",
		      "enforceSizeThreshold": 30000,
		      "maxAsyncRequests": Infinity,
		      "maxInitialRequests": Infinity,
		      "minChunks": 1,
		      "minRemainingSize": undefined,
		      "minSize": 10000,
		    },
		  },
		  "output": {
		    "assetModuleFilename": "[hash][ext][query]",
		    "chunkFilename": "[name].js",
		    "cssChunkFilename": "[name].css",
		    "cssFilename": "[name].css",
		    "enabledLibraryTypes": [],
		    "filename": "[name].js",
		    "globalObject": "self",
		    "importFunctionName": "import",
		    "library": undefined,
		    "path": "<cwd>/dist",
		    "publicPath": "auto",
		    "strictModuleErrorHandling": false,
		    "uniqueName": "@rspack/core",
		  },
		  "plugins": [],
		  "resolve": {
		    "browserField": true,
		    "extensions": [
		      ".tsx",
		      ".jsx",
		      ".ts",
		      ".js",
		      ".json",
		      ".d.ts",
		    ],
		    "mainFields": [
		      "browser",
		      "module",
		      "main",
		    ],
		    "mainFiles": [
		      "index",
		    ],
		    "modules": [
		      "node_modules",
		    ],
		  },
		  "snapshot": {
		    "module": {
		      "hash": false,
		      "timestamp": true,
		    },
		    "resolve": {
		      "hash": false,
		      "timestamp": true,
		    },
		  },
		  "stats": {},
		  "target": "web",
		  "watch": false,
		  "watchOptions": {},
		}
	`);
	});

	const test = (name, options, fn, before, after) => {
		it(`should generate the correct defaults from ${name}`, () => {
			if (!("mode" in options)) options.mode = "none";
			try {
				if (before) before();
				const result = getDefaultConfig(options);

				const diff = stripAnsi(
					jestDiff(baseConfig, result, { expand: false, contextLines: 0 })
				);

				fn(expect(new Diff(diff)), expect(result));
			} finally {
				if (after) after();
			}
		});
	};

	test("empty config", {}, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("none mode", { mode: "none" }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("no mode provided", { mode: undefined }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-         "localIdentName": "[path][name][ext]__[local]",
		+         "localIdentName": "[hash]",
		@@ ... @@
		-     "minifyOptions": undefined,
		+     "minifyOptions": Object {
		+       "dropConsole": false,
		+       "passes": 1,
		+       "pureFuncs": Array [],
		+     },
		@@ ... @@
		-     "treeShaking": false,
		+     "treeShaking": true,
		@@ ... @@
		-   "mode": "none",
		+   "mode": undefined,
		@@ ... @@
		-     "minimize": false,
		+     "minimize": true,
		@@ ... @@
		-     "moduleIds": "named",
		+     "moduleIds": "deterministic",
		@@ ... @@
		-     "sideEffects": "flag",
		+     "sideEffects": true,
		@@ ... @@
		-       "enforceSizeThreshold": 30000,
		-       "maxAsyncRequests": Infinity,
		-       "maxInitialRequests": Infinity,
		+       "enforceSizeThreshold": 50000,
		+       "maxAsyncRequests": 30,
		+       "maxInitialRequests": 30,
		@@ ... @@
		-       "minSize": 10000,
		+       "minSize": 20000,
		@@ ... @@
		-       "hash": false,
		+       "hash": true,
		@@ ... @@
		-       "hash": false,
		+       "hash": true,
	`)
	);
	test("production", { mode: "production" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-         "localIdentName": "[path][name][ext]__[local]",
		+         "localIdentName": "[hash]",
		@@ ... @@
		-     "minifyOptions": undefined,
		+     "minifyOptions": Object {
		+       "dropConsole": false,
		+       "passes": 1,
		+       "pureFuncs": Array [],
		+     },
		@@ ... @@
		-     "treeShaking": false,
		+     "treeShaking": true,
		@@ ... @@
		-   "mode": "none",
		+   "mode": "production",
		@@ ... @@
		-     "minimize": false,
		+     "minimize": true,
		@@ ... @@
		-     "moduleIds": "named",
		+     "moduleIds": "deterministic",
		@@ ... @@
		-     "sideEffects": "flag",
		+     "sideEffects": true,
		@@ ... @@
		-       "enforceSizeThreshold": 30000,
		-       "maxAsyncRequests": Infinity,
		-       "maxInitialRequests": Infinity,
		+       "enforceSizeThreshold": 50000,
		+       "maxAsyncRequests": 30,
		+       "maxInitialRequests": 30,
		@@ ... @@
		-       "minSize": 10000,
		+       "minSize": 20000,
		@@ ... @@
		-       "hash": false,
		+       "hash": true,
		@@ ... @@
		-       "hash": false,
		+       "hash": true,
	`)
	);
	test("development", { mode: "development" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": true,
		@@ ... @@
		-   "mode": "none",
		+   "mode": "development",
		@@ ... @@
		-       "minRemainingSize": undefined,
		+       "minRemainingSize": 0,
	`)
	);
	/**
	 * not support yet
	 */
	test("sync wasm", { experiments: { syncWebAssembly: true } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+     "syncWebAssembly": true,
	`)
	);
	/**
	 * not support yet
	 */
	test("output module", { experiments: { outputModule: true } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+     "outputModule": true,
	`)
	);
	/**
	 * not support yet
	 */
	test("async wasm", { experiments: { asyncWebAssembly: true } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+     "asyncWebAssembly": true,
	`)
	);
	test(
		"both wasm",
		{ experiments: { syncWebAssembly: true, asyncWebAssembly: true } },
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			+     "asyncWebAssembly": true,
			@@ ... @@
			+     "syncWebAssembly": true,
		`)
	);
	test("const filename", { output: { filename: "bundle.js" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkFilename": "[name].js",
		-     "cssChunkFilename": "[name].css",
		-     "cssFilename": "[name].css",
		+     "chunkFilename": "[id].bundle.js",
		+     "cssChunkFilename": "[id].bundle.css",
		+     "cssFilename": "bundle.css",
		@@ ... @@
		-     "filename": "[name].js",
		+     "filename": "bundle.js",
	`)
	);
	test("function filename", { output: { filename: () => "bundle.js" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkFilename": "[name].js",
		-     "cssChunkFilename": "[name].css",
		-     "cssFilename": "[name].css",
		+     "chunkFilename": "[id].js",
		+     "cssChunkFilename": "[id].css",
		+     "cssFilename": "[id].css",
		@@ ... @@
		-     "filename": "[name].js",
		+     "filename": [Function filename],
	`)
	);
	test("library", { output: { library: ["myLib", "awesome"] } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "enabledLibraryTypes": Array [],
		+     "enabledLibraryTypes": Array [
		+       "var",
		+     ],
		@@ ... @@
		-     "library": undefined,
		+     "library": Object {
		+       "auxiliaryComment": undefined,
		+       "export": undefined,
		+       "name": Array [
		+         "myLib",
		+         "awesome",
		+       ],
		+       "type": "var",
		+       "umdNamedDefine": undefined,
		+     },
	`)
	);
	test(
		"library contains [name] placeholder",
		{
			output: {
				library: ["myLib", "[name]"]
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "auxiliaryComment": undefined,
			+       "export": undefined,
			+       "name": Array [
			+         "myLib",
			+         "[name]",
			+       ],
			+       "type": "var",
			+       "umdNamedDefine": undefined,
			+     },
		`)
	);
	test(
		"library.name contains [name] placeholder",
		{
			output: {
				library: {
					name: ["my[name]Lib", "[name]", "lib"],
					type: "var"
				}
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "auxiliaryComment": undefined,
			+       "export": undefined,
			+       "name": Array [
			+         "my[name]Lib",
			+         "[name]",
			+         "lib",
			+       ],
			+       "type": "var",
			+       "umdNamedDefine": undefined,
			+     },
		`)
	);
	test(
		"library.name.root contains [name] placeholder",
		{
			output: {
				library: {
					name: {
						root: ["[name]", "myLib"]
					},
					type: "var"
				}
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "auxiliaryComment": undefined,
			+       "export": undefined,
			+       "name": Object {
			+         "root": Array [
			+           "[name]",
			+           "myLib",
			+         ],
			+       },
			+       "type": "var",
			+       "umdNamedDefine": undefined,
			+     },
		`)
	);
	test(
		"library.name.root contains escaped placeholder",
		{
			output: {
				library: {
					name: {
						root: ["[\\name\\]", "my[\\name\\]Lib[name]", "[\\name\\]"]
					},
					type: "var"
				}
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "auxiliaryComment": undefined,
			+       "export": undefined,
			+       "name": Object {
			+         "root": Array [
			+           "[\\\\name\\\\]",
			+           "my[\\\\name\\\\]Lib[name]",
			+           "[\\\\name\\\\]",
			+         ],
			+       },
			+       "type": "var",
			+       "umdNamedDefine": undefined,
			+     },
		`)
	);
	test("target node", { target: "node" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "node": false,
		+     "node": true,
		@@ ... @@
		-     "__dirname": "warn-mock",
		-     "__filename": "warn-mock",
		-     "global": "warn",
		+     "__dirname": "eval-only",
		+     "__filename": "eval-only",
		+     "global": false,
		@@ ... @@
		-     "globalObject": "self",
		+     "globalObject": "global",
		@@ ... @@
		-     "publicPath": "auto",
		+     "publicPath": "",
		@@ ... @@
		-     "browserField": true,
		+     "browserField": false,
		@@ ... @@
		-       "browser",
		@@ ... @@
		-   "target": "web",
		+   "target": "node",
	`)
	);
	test("target webworker", { target: "webworker" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "target": "web",
		+   "target": "webworker",
	`)
	);
	test("target electron-main", { target: "electron-main" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "node": false,
		+     "node": true,
		@@ ... @@
		-     "__dirname": "warn-mock",
		-     "__filename": "warn-mock",
		-     "global": "warn",
		+     "__dirname": "eval-only",
		+     "__filename": "eval-only",
		+     "global": false,
		@@ ... @@
		-     "globalObject": "self",
		+     "globalObject": "global",
		@@ ... @@
		-     "publicPath": "auto",
		+     "publicPath": "",
		@@ ... @@
		-     "browserField": true,
		+     "browserField": false,
		@@ ... @@
		-       "browser",
		@@ ... @@
		-   "target": "web",
		+   "target": "electron-main",
	`)
	);
	test("target electron-main", { target: "electron-preload" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "node": false,
		+     "node": true,
		@@ ... @@
		-     "__dirname": "warn-mock",
		-     "__filename": "warn-mock",
		-     "global": "warn",
		+     "__dirname": "eval-only",
		+     "__filename": "eval-only",
		+     "global": false,
		@@ ... @@
		-     "globalObject": "self",
		+     "globalObject": "global",
		@@ ... @@
		-     "publicPath": "auto",
		+     "publicPath": "",
		@@ ... @@
		-     "browserField": true,
		+     "browserField": false,
		@@ ... @@
		-       "browser",
		@@ ... @@
		-   "target": "web",
		+   "target": "electron-preload",
	`)
	);
	test("records", { recordsPath: "some-path" }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("ecmaVersion", { output: { ecmaVersion: 2020 } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("single runtimeChunk", { optimization: { runtimeChunk: "single" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "runtimeChunk": false,
		+     "runtimeChunk": Object {
		+       "name": [Function name],
		+     },
	`)
	);
	test(
		"single runtimeChunk",
		{ optimization: { runtimeChunk: "multiple" } },
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "runtimeChunk": false,
			+     "runtimeChunk": Object {
			+       "name": [Function name],
			+     },
		`)
	);
	test("single runtimeChunk", { optimization: { runtimeChunk: true } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "runtimeChunk": false,
		+     "runtimeChunk": Object {
		+       "name": [Function name],
		+     },
	`)
	);
	test("cache true", { cache: true }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": true,
	`)
	);
	test("cache filesystem", { cache: { type: "filesystem" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": Object {
		+     "type": "filesystem",
		+   },
	`)
	);
	test(
		"cache filesystem development",
		{ mode: "development", cache: { type: "filesystem" } },
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-   "cache": false,
			+   "cache": Object {
			+     "type": "filesystem",
			+   },
			@@ ... @@
			-   "mode": "none",
			+   "mode": "development",
			@@ ... @@
			-       "minRemainingSize": undefined,
			+       "minRemainingSize": 0,
		`)
	);

	// TODO: options.node = false
	// test(
	// 	"disable",
	// 	{
	// 		cache: false,
	// 		node: false,
	// 		amd: false,
	// 		optimization: { splitChunks: false }
	// 	},
	// 	e =>
	// 		e.toMatchInlineSnapshot(`
	// 		- Expected
	// 		+ Received

	// 		@@ ... @@
	// 		-   "cache": undefined,
	// 		+   "cache": false,
	// 		@@ ... @@
	// 		-   "node": Object {},
	// 		+   "node": false,
	// 		@@ ... @@
	// 		-     "splitChunks": Object {
	// 		-       "cacheGroups": Object {},
	// 		-     },
	// 		+     "splitChunks": false,
	// 	`)
	// );

	test(
		"uniqueName",
		{
			output: {
				uniqueName: "@@@Hello World!",
				trustedTypes: true
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "uniqueName": "@rspack/core",
			+     "uniqueName": "@@@Hello World!",
		`)
	);

	test("stats true", { stats: true }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "stats": Object {},
		+   "stats": Object {
		+     "preset": "normal",
		+   },
	`)
	);

	test("stats false", { stats: false }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "stats": Object {},
		+   "stats": Object {
		+     "preset": "none",
		+   },
	`)
	);

	test("stats string", { stats: "minimal" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "stats": Object {},
		+   "stats": Object {
		+     "preset": "minimal",
		+   },
	`)
	);

	test(
		"browserslist",
		{ context: path.resolve(__dirname, "fixtures/browserslist") },
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-   "context": "<cwd>",
			+   "context": "<cwd>/tests/fixtures/browserslist",
			@@ ... @@
			-     "uniqueName": "@rspack/core",
			+     "uniqueName": "browserslist-test",
		`)
	);

	test(
		"non-root directory",
		{
			cache: {
				type: "filesystem"
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-   "cache": false,
			-   "context": "<cwd>",
			+   "cache": Object {
			+     "type": "filesystem",
			+   },
			+   "context": "<cwd>/tests/fixtures",
			@@ ... @@
			-     "path": "<cwd>/dist",
			+     "path": "<cwd>/tests/fixtures/dist",
			@@ ... @@
			-     "uniqueName": "@rspack/core",
			+     "uniqueName": "",
		`),
		() => {
			process.chdir(path.resolve(__dirname, "fixtures"));
		},
		() => {
			process.chdir(cwd);
		}
	);

	test(
		"array defaults",
		{
			output: {
				enabledChunkLoadingTypes: ["require", "..."],
				enabledWasmLoadingTypes: ["...", "async-node"]
			}
		},
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);

	test(
		"experiments.futureDefaults",
		{
			experiments: {
				futureDefaults: true
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			+     "futureDefaults": true,
		`)
	);

	test(
		"experiments.futureDefaults w/ experiments.css disabled",
		{
			experiments: {
				css: false,
				futureDefaults: true
			}
		},
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			+     "css": false,
			+     "futureDefaults": true,
		`)
	);
});
export {};
