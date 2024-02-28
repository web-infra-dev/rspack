// @ts-nocheck
const path = require("path");
const jestDiff = require("jest-diff").diff;
const stripAnsi = require("strip-ansi");
import rspack from "../src";
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
	config = rspack.config.getNormalizedRspackOptions(config);
	rspack.config.applyRspackOptionsDefaults(config);
	process.chdir(cwd);
	// make snapshot stable
	config.experiments.rspackFuture.bundlerInfo.version = "$version$";
	return config;
};

describe("snapshots", () => {
	const baseConfig = getDefaultConfig({ mode: "none" });

	it("should have the correct base config", () => {
		expect(baseConfig).toMatchSnapshot();
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
		-   "mode": "none",
		+   "mode": undefined,
		@@ ... @@
		-     "chunkIds": "named",
		+     "chunkIds": "deterministic",
		@@ ... @@
		-     "innerGraph": false,
		-     "mangleExports": false,
		+     "innerGraph": true,
		+     "mangleExports": true,
		@@ ... @@
		-     "minimize": false,
		+     "minimize": true,
		@@ ... @@
		-     "moduleIds": "named",
		-     "nodeEnv": false,
		+     "moduleIds": "deterministic",
		+     "nodeEnv": "production",
		@@ ... @@
		-     "realContentHash": false,
		+     "realContentHash": true,
		@@ ... @@
		-     "sideEffects": "flag",
		+     "sideEffects": true,
		@@ ... @@
		-       "hidePathInfo": false,
		-       "maxAsyncRequests": Infinity,
		-       "maxInitialRequests": Infinity,
		+       "hidePathInfo": true,
		+       "maxAsyncRequests": 30,
		+       "maxInitialRequests": 30,
		@@ ... @@
		-       "minSize": 10000,
		+       "minSize": 20000,
		@@ ... @@
		-     "usedExports": false,
		+     "usedExports": true,
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
		-   "mode": "none",
		+   "mode": "production",
		@@ ... @@
		-     "chunkIds": "named",
		+     "chunkIds": "deterministic",
		@@ ... @@
		-     "innerGraph": false,
		-     "mangleExports": false,
		+     "innerGraph": true,
		+     "mangleExports": true,
		@@ ... @@
		-     "minimize": false,
		+     "minimize": true,
		@@ ... @@
		-     "moduleIds": "named",
		-     "nodeEnv": false,
		+     "moduleIds": "deterministic",
		+     "nodeEnv": "production",
		@@ ... @@
		-     "realContentHash": false,
		+     "realContentHash": true,
		@@ ... @@
		-     "sideEffects": "flag",
		+     "sideEffects": true,
		@@ ... @@
		-       "hidePathInfo": false,
		-       "maxAsyncRequests": Infinity,
		-       "maxInitialRequests": Infinity,
		+       "hidePathInfo": true,
		+       "maxAsyncRequests": 30,
		+       "maxInitialRequests": 30,
		@@ ... @@
		-       "minSize": 10000,
		+       "minSize": 20000,
		@@ ... @@
		-     "usedExports": false,
		+     "usedExports": true,
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
		-     "nodeEnv": false,
		+     "nodeEnv": "development",
		@@ ... @@
		-       "production",
		+       "development",
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
		@@ ... @@
		-   "externalsType": "var",
		+   "externalsType": "module",
		@@ ... @@
		-     "chunkFilename": "[name].js",
		+     "chunkFilename": "[name].mjs",
		@@ ... @@
		-     "filename": "[name].js",
		+     "filename": "[name].mjs",
		@@ ... @@
		-     "hotUpdateChunkFilename": "[id].[fullhash].hot-update.js",
		+     "hotUpdateChunkFilename": "[id].[fullhash].hot-update.mjs",
		@@ ... @@
		-     "iife": true,
		+     "iife": false,
		@@ ... @@
		-     "module": false,
		+     "module": true,
		@@ ... @@
		-     "scriptType": false,
		+     "scriptType": "module",
	`)
	);

	test("async wasm", { experiments: { asyncWebAssembly: true } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "asyncWebAssembly": false,
		+     "asyncWebAssembly": true,
		@@ ... @@
		+       },
		+       Object {
		+         "rules": Array [
		+           Object {
		+             "descriptionData": Object {
		+               "type": "module",
		+             },
		+             "resolve": Object {
		+               "fullySpecified": true,
		+             },
		+           },
		+         ],
		+         "test": /\\.wasm$/i,
		+         "type": "webassembly/async",
		+       },
		+       Object {
		+         "mimetype": "application/wasm",
		+         "rules": Array [
		+           Object {
		+             "descriptionData": Object {
		+               "type": "module",
		+             },
		+             "resolve": Object {
		+               "fullySpecified": true,
		+             },
		+           },
		+         ],
		+         "type": "webassembly/async",
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
			-     "asyncWebAssembly": false,
			+     "asyncWebAssembly": true,
			@@ ... @@
			+     "syncWebAssembly": true,
			@@ ... @@
			+       },
			+       Object {
			+         "rules": Array [
			+           Object {
			+             "descriptionData": Object {
			+               "type": "module",
			+             },
			+             "resolve": Object {
			+               "fullySpecified": true,
			+             },
			+           },
			+         ],
			+         "test": /\\.wasm$/i,
			+         "type": "webassembly/async",
			+       },
			+       Object {
			+         "mimetype": "application/wasm",
			+         "rules": Array [
			+           Object {
			+             "descriptionData": Object {
			+               "type": "module",
			+             },
			+             "resolve": Object {
			+               "fullySpecified": true,
			+             },
			+           },
			+         ],
			+         "type": "webassembly/async",
		`)
	);
	test("const filename", { output: { filename: "bundle.js" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkFilename": "[name].js",
		+     "chunkFilename": "[id].bundle.js",
		@@ ... @@
		-     "cssChunkFilename": "[name].css",
		-     "cssFilename": "[name].css",
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
		+     "chunkFilename": "[id].js",
		@@ ... @@
		-     "cssChunkFilename": "[name].css",
		-     "cssFilename": "[name].css",
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
		-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
		+     "chunkLoadingGlobal": "webpackChunkmyLib_awesome",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/core",
		+     "devtoolNamespace": "myLib.awesome",
		@@ ... @@
		-     "enabledLibraryTypes": Array [],
		+     "enabledLibraryTypes": Array [
		+       "var",
		+     ],
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
		+     "hotUpdateGlobal": "webpackHotUpdatemyLib_awesome",
		@@ ... @@
		-     "library": undefined,
		+     "library": Object {
		+       "amdContainer": undefined,
		+       "auxiliaryComment": undefined,
		+       "export": undefined,
		+       "name": Array [
		+         "myLib",
		+         "awesome",
		+       ],
		+       "type": "var",
		+       "umdNamedDefine": undefined,
		+     },
		@@ ... @@
		-     "uniqueName": "@rspack/core",
		+     "uniqueName": "myLib.awesome",
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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
			+     "chunkLoadingGlobal": "webpackChunkmyLib",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/core",
			+     "devtoolNamespace": "myLib",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
			+     "hotUpdateGlobal": "webpackHotUpdatemyLib",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "amdContainer": undefined,
			+       "auxiliaryComment": undefined,
			+       "export": undefined,
			+       "name": Array [
			+         "myLib",
			+         "[name]",
			+       ],
			+       "type": "var",
			+       "umdNamedDefine": undefined,
			+     },
			@@ ... @@
			-     "uniqueName": "@rspack/core",
			+     "uniqueName": "myLib",
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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
			+     "chunkLoadingGlobal": "webpackChunkmyLib_lib",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/core",
			+     "devtoolNamespace": "myLib.lib",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
			+     "hotUpdateGlobal": "webpackHotUpdatemyLib_lib",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "amdContainer": undefined,
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
			@@ ... @@
			-     "uniqueName": "@rspack/core",
			+     "uniqueName": "myLib.lib",
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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
			+     "chunkLoadingGlobal": "webpackChunkmyLib",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/core",
			+     "devtoolNamespace": "myLib",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
			+     "hotUpdateGlobal": "webpackHotUpdatemyLib",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "amdContainer": undefined,
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
			@@ ... @@
			-     "uniqueName": "@rspack/core",
			+     "uniqueName": "myLib",
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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
			+     "chunkLoadingGlobal": "webpackChunk_name_my_name_Lib_name_",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/core",
			+     "devtoolNamespace": "[name].my[name]Lib.[name]",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
			+     "hotUpdateGlobal": "webpackHotUpdate_name_my_name_Lib_name_",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "amdContainer": undefined,
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
			@@ ... @@
			-     "uniqueName": "@rspack/core",
			+     "uniqueName": "[name].my[name]Lib.[name]",
		`)
	);
	test("target node", { target: "node" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "node": false,
		-     "web": true,
		+     "node": true,
		+     "web": false,
		@@ ... @@
		-     "__dirname": "warn-mock",
		-     "__filename": "warn-mock",
		-     "global": "warn",
		+     "__dirname": "eval-only",
		+     "__filename": "eval-only",
		+     "global": false,
		@@ ... @@
		-     "chunkFormat": "array-push",
		-     "chunkLoading": "jsonp",
		+     "chunkFormat": "commonjs",
		+     "chunkLoading": "require",
		@@ ... @@
		-       "jsonp",
		-       "import-scripts",
		+       "require",
		@@ ... @@
		-       "fetch",
		+       "async-node",
		@@ ... @@
		-     "globalObject": "self",
		+     "globalObject": "global",
		@@ ... @@
		-     "publicPath": "auto",
		+     "publicPath": "",
		@@ ... @@
		-     "wasmLoading": "fetch",
		+     "wasmLoading": "async-node",
		@@ ... @@
		-     "workerChunkLoading": "import-scripts",
		+     "workerChunkLoading": "require",
		@@ ... @@
		-     "workerWasmLoading": "fetch",
		+     "workerWasmLoading": "async-node",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-       "browser",
		+       "node",
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
		-     "chunkLoading": "jsonp",
		+     "chunkLoading": "import-scripts",
		@@ ... @@
		-       "jsonp",
		@@ ... @@
		+       "worker",
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
		-     "electron": false,
		-     "electronMain": false,
		+     "electron": true,
		+     "electronMain": true,
		@@ ... @@
		-     "node": false,
		-     "web": true,
		+     "node": true,
		+     "web": false,
		@@ ... @@
		-     "__dirname": "warn-mock",
		-     "__filename": "warn-mock",
		-     "global": "warn",
		+     "__dirname": "eval-only",
		+     "__filename": "eval-only",
		+     "global": false,
		@@ ... @@
		-     "chunkFormat": "array-push",
		-     "chunkLoading": "jsonp",
		+     "chunkFormat": "commonjs",
		+     "chunkLoading": "require",
		@@ ... @@
		-       "jsonp",
		-       "import-scripts",
		+       "require",
		@@ ... @@
		-       "fetch",
		+       "async-node",
		@@ ... @@
		-     "globalObject": "self",
		+     "globalObject": "global",
		@@ ... @@
		-     "publicPath": "auto",
		+     "publicPath": "",
		@@ ... @@
		-     "wasmLoading": "fetch",
		+     "wasmLoading": "async-node",
		@@ ... @@
		-     "workerChunkLoading": "import-scripts",
		+     "workerChunkLoading": "require",
		@@ ... @@
		-     "workerWasmLoading": "fetch",
		+     "workerWasmLoading": "async-node",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-       "browser",
		+       "node",
		+       "electron",
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
		-     "electron": false,
		+     "electron": true,
		@@ ... @@
		-     "electronPreload": false,
		+     "electronPreload": true,
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
		-     "chunkFormat": "array-push",
		-     "chunkLoading": "jsonp",
		+     "chunkFormat": "commonjs",
		+     "chunkLoading": "require",
		@@ ... @@
		-       "jsonp",
		-       "import-scripts",
		+       "require",
		@@ ... @@
		-       "fetch",
		+       "async-node",
		@@ ... @@
		-     "globalObject": "self",
		+     "globalObject": "global",
		@@ ... @@
		-     "publicPath": "auto",
		+     "publicPath": "",
		@@ ... @@
		-     "wasmLoading": "fetch",
		+     "wasmLoading": "async-node",
		@@ ... @@
		-     "workerChunkLoading": "import-scripts",
		+     "workerChunkLoading": "require",
		@@ ... @@
		-     "workerWasmLoading": "fetch",
		+     "workerWasmLoading": "async-node",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		+       "node",
		@@ ... @@
		+       "electron",
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
			-     "nodeEnv": false,
			+     "nodeEnv": "development",
			@@ ... @@
			-       "production",
			+       "development",
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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
			+     "chunkLoadingGlobal": "webpackChunk_Hello_World_",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/core",
			+     "devtoolNamespace": "@@@Hello World!",
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
			+     "hotUpdateGlobal": "webpackHotUpdate_Hello_World_",
			@@ ... @@
			-     "trustedTypes": undefined,
			-     "uniqueName": "@rspack/core",
			+     "trustedTypes": Object {
			+       "policyName": "@@@Hello_World_",
			+     },
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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
			+     "chunkLoadingGlobal": "webpackChunkbrowserslist_test",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/core",
			+     "devtoolNamespace": "browserslist-test",
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
			+     "hotUpdateGlobal": "webpackHotUpdatebrowserslist_test",
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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
			+     "chunkLoadingGlobal": "webpackChunk",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/core",
			+     "devtoolNamespace": "",
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
			+     "hotUpdateGlobal": "webpackHotUpdate",
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
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			+       "require",
			@@ ... @@
			+       "async-node",
		`)
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
			@@ ... @@
			-     "hashDigestLength": 20,
			-     "hashFunction": "md4",
			+     "hashDigestLength": 16,
			+     "hashFunction": "xxhash64",
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
			-     "css": true,
			+     "css": false,
			+     "futureDefaults": true,
			@@ ... @@
			-       },
			-       Object {
			-         "oneOf": Array [
			-           Object {
			-             "resolve": Object {
			-               "fullySpecified": true,
			-             },
			-             "test": /\\.module\\.css$/i,
			-             "type": "css/module",
			-           },
			-           Object {
			-             "resolve": Object {
			-               "fullySpecified": true,
			-               "preferRelative": true,
			-             },
			-             "type": "css",
			-           },
			-         ],
			-         "test": /\\.css$/i,
			@@ ... @@
			-         "mimetype": "text/css+module",
			-         "resolve": Object {
			-           "fullySpecified": true,
			-         },
			-         "type": "css/module",
			-       },
			-       Object {
			-         "mimetype": "text/css",
			-         "resolve": Object {
			-           "fullySpecified": true,
			-           "preferRelative": true,
			-         },
			-         "type": "css",
			-       },
			-       Object {
			@@ ... @@
			-         "css",
			@@ ... @@
			-     "hashDigestLength": 20,
			-     "hashFunction": "md4",
			+     "hashDigestLength": 16,
			+     "hashFunction": "xxhash64",
		`)
	);
});
