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
    "decorator": {
      "emitMetadata": true,
      "legacy": true,
      "useDefineForClassFields": true,
    },
    "html": [],
  },
  "context": "<cwd>",
  "devServer": undefined,
  "devtool": "",
  "entry": {
    "main": [
      "<cwd>/src/index.js",
    ],
  },
  "externals": {},
  "externalsType": "",
  "infrastructureLogging": {},
  "mode": "none",
  "module": {
    "rules": [],
  },
  "output": {},
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
    "tsConfigPath": "",
  },
  "stats": {
    "colors": false,
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
		+   "mode": "production",
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
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+   "experiments": Object {
		+     "syncWebAssembly": true,
		+   },
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
		+   "experiments": Object {
		+     "outputModule": true,
		+   },
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
		+   "experiments": Object {
		+     "asyncWebAssembly": true,
		+   },
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
			+   "experiments": Object {
			+     "asyncWebAssembly": true,
			+     "syncWebAssembly": true,
			+   },
		`)
	);
	test("const filename", { output: { filename: "bundle.js" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "output": Object {},
		+   "output": Object {
		+     "filename": "bundle.js",
		+   },
	`)
	);
	test("function filename", { output: { filename: () => "bundle.js" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "output": Object {},
		+   "output": Object {
		+     "filename": [Function filename],
		+   },
	`)
	);
	test("library", { output: { library: ["myLib", "awesome"] } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "output": Object {},
		+   "output": Object {
		+     "library": Array [
		+       "myLib",
		+       "awesome",
		+     ],
		+   },
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
			-   "output": Object {},
			+   "output": Object {
			+     "library": Array [
			+       "myLib",
			+       "[name]",
			+     ],
			+   },
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
			-   "output": Object {},
			+   "output": Object {
			+     "library": Object {
			+       "name": Array [
			+         "my[name]Lib",
			+         "[name]",
			+         "lib",
			+       ],
			+       "type": "var",
			+     },
			+   },
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
			-   "output": Object {},
			+   "output": Object {
			+     "library": Object {
			+       "name": Object {
			+         "root": Array [
			+           "[name]",
			+           "myLib",
			+         ],
			+       },
			+       "type": "var",
			+     },
			+   },
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
			-   "output": Object {},
			+   "output": Object {
			+     "library": Object {
			+       "name": Object {
			+         "root": Array [
			+           "[\\\\name\\\\]",
			+           "my[\\\\name\\\\]Lib[name]",
			+           "[\\\\name\\\\]",
			+         ],
			+       },
			+       "type": "var",
			+     },
			+   },
		`)
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
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+   "recordsPath": "some-path",
	`)
	);
	test("ecmaVersion", { output: { ecmaVersion: 2020 } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "output": Object {},
		+   "output": Object {
		+     "ecmaVersion": 2020,
		+   },
	`)
	);
	test("single runtimeChunk", { optimization: { runtimeChunk: "single" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+   "optimization": Object {
		+     "runtimeChunk": "single",
		+   },
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
			+   "optimization": Object {
			+     "runtimeChunk": "multiple",
			+   },
		`)
	);
	test("single runtimeChunk", { optimization: { runtimeChunk: true } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+   "optimization": Object {
		+     "runtimeChunk": true,
		+   },
	`)
	);
	test("cache true", { cache: true }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+   "cache": true,
	`)
	);
	test("cache filesystem", { cache: { type: "filesystem" } }, e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
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
			+   "cache": Object {
			+     "type": "filesystem",
			+   },
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
		e =>
			e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			+   "amd": false,
			@@ ... @@
			+   "cache": false,
			@@ ... @@
			+   },
			+   "node": false,
			+   "optimization": Object {
			+     "splitChunks": false,
		`)
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
			-   "output": Object {},
			+   "output": Object {
			+     "trustedTypes": true,
			+     "uniqueName": "@@@Hello World!",
			+   },
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
			-       "<cwd>/src/index.js",
			+       "<cwd>/tests/fixtures/browserslist/src/index.js",
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
			+   "cache": Object {
			+     "type": "filesystem",
			+   },
			+   "context": "<cwd>/tests/fixtures",
			@@ ... @@
			-       "<cwd>/src/index.js",
			+       "<cwd>/tests/fixtures/src/index.js",
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
			-   "output": Object {},
			+   "output": Object {
			+     "enabledChunkLoadingTypes": Array [
			+       "require",
			+       "...",
			+     ],
			+     "enabledWasmLoadingTypes": Array [
			+       "...",
			+       "async-node",
			+     ],
			+   },
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
			+   "experiments": Object {
			+     "futureDefaults": true,
			+   },
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
			+   "experiments": Object {
			+     "css": false,
			+     "futureDefaults": true,
			+   },
		`)
	);
});
export {};
