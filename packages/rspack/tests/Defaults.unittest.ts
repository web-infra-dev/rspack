// @ts-nocheck
const path = require("path");
const jestDiff = require("jest-diff").diff;
const stripAnsi = require("strip-ansi");
import { getNormalizedRspackOptions } from "../src";
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
	process.chdir(cwd);
	return config;
};

describe("snapshots", () => {
	const baseConfig = getDefaultConfig({ mode: "none" });

	it("should have the correct base config", () => {
		expect(baseConfig).toMatchInlineSnapshot(`
		{
		  "builtins": {
		    "browserslist": [],
		  },
		  "context": "<cwd>",
		  "define": {},
		  "devServer": {
		    "devMiddleware": {},
		    "hmr": false,
		    "liveReload": true,
		    "open": true,
		    "port": 8080,
		    "static": {
		      "directory": "<cwd>/dist",
		      "watch": {},
		    },
		    "webSocketServer": {},
		  },
		  "devtool": "",
		  "entry": {},
		  "external": {},
		  "externalType": "",
		  "mode": "none",
		  "module": {
		    "rules": [],
		  },
		  "output": {
		    "assetModuleFilename": undefined,
		    "chunkFilename": undefined,
		    "filename": undefined,
		    "path": undefined,
		    "publicPath": undefined,
		    "uniqueName": undefined,
		  },
		  "plugins": [],
		  "resolve": {
		    "alias": {},
		    "browserField": true,
		    "conditionNames": [
		      "module",
		      "import",
		    ],
		    "extensions": [
		      ".tsx",
		      ".jsx",
		      ".ts",
		      ".js",
		      ".json",
		      ".d.ts",
		    ],
		    "mainFields": [
		      "module",
		      "main",
		    ],
		    "mainFiles": [
		      "index",
		    ],
		    "preferRelative": false,
		  },
		  "target": [
		    "web",
		  ],
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
		-   "mode": "none",
		+   "mode": "development",
	`)
	);
	test("production", { mode: "production" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "mode": "none",
		+   "mode": "production",
	`)
	);
	test("development", { mode: "development" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "mode": "none",
		+   "mode": "development",
	`)
	);
	/**
	 * not support yet
	 */
	test("sync wasm", { experiments: { syncWebAssembly: true } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	/**
	 * not support yet
	 */
	test("output module", { experiments: { outputModule: true } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	/**
	 * not support yet
	 */
	test("async wasm", { experiments: { asyncWebAssembly: true } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test(
		"both wasm",
		{ experiments: { syncWebAssembly: true, asyncWebAssembly: true } },
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("const filename", { output: { filename: "bundle.js" } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("function filename", { output: { filename: () => "bundle.js" } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("library", { output: { library: ["myLib", "awesome"] } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test(
		"library contains [name] placeholder",
		{
			output: {
				library: ["myLib", "[name]"]
			}
		},
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
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
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
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
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
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
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("target node", { target: "node" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "web",
		+     "node",
	`)
	);
	test("target webworker", { target: "webworker" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "web",
		+     "webworker",
	`)
	);
	test("target electron-main", { target: "electron-main" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "web",
		+     "electron-main",
	`)
	);
	test("target electron-main", { target: "electron-preload" }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "web",
		+     "electron-preload",
	`)
	);
	test("records", { recordsPath: "some-path" }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("ecmaVersion", { output: { ecmaVersion: 2020 } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("single runtimeChunk", { optimization: { runtimeChunk: "single" } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test(
		"single runtimeChunk",
		{ optimization: { runtimeChunk: "multiple" } },
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("single runtimeChunk", { optimization: { runtimeChunk: true } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("cache true", { cache: true }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test("cache filesystem", { cache: { type: "filesystem" } }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
	test(
		"cache filesystem development",
		{ mode: "development", cache: { type: "filesystem" } },
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-   "mode": "none",
			+   "mode": "development",
		`)
	);

	test(
		"disable",
		{
			cache: false,
			node: false,
			amd: false,
			optimization: { splitChunks: false }
		},
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);

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
			-     "uniqueName": undefined,
			+     "uniqueName": "@@@Hello World!",
		`)
	);

	test("stats true", { stats: true }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);

	test("stats false", { stats: false }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);

	test("stats string", { stats: "minimal" }, e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);

	test(
		"browserslist",
		{ context: path.resolve(__dirname, "fixtures/browserslist") },
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "browserslist": Array [],
			+     "browserslist": Array [
			+       "ie >= 9",
			+     ],
			@@ ... @@
			-   "context": "<cwd>",
			+   "context": "<cwd>/tests/fixtures/browserslist",
			@@ ... @@
			-       "directory": "<cwd>/dist",
			+       "directory": "<cwd>/tests/fixtures/browserslist/dist",
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
			-   "context": "<cwd>",
			+   "context": "<cwd>/tests/fixtures",
			@@ ... @@
			-       "directory": "<cwd>/dist",
			+       "directory": "<cwd>/tests/fixtures/dist",
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
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);

	test(
		"experiments.futureDefaults w/ experiments.css disabled",
		{
			experiments: {
				css: false,
				futureDefaults: true
			}
		},
		e => e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
	);
});
export {};
