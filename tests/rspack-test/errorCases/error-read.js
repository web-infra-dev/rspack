let errors = [];

/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description:
			"Testing map function on errors and warnings: test map of errors",
		options() {
			errors = [];
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test errors map", compilation => {
							errors = compilation.errors.map((item, index) => {
								item.index = index;
								return item;
							});
						});
					}
				]
			};
		},
		async check() {
			expect(errors).toMatchInlineSnapshot(`
			Array [
			  Object {
			  "index": 0,
			  "loc": Object {
			    "end": Object {
			      "column": 33,
			      "line": 1,
			    },
			    "start": Object {
			      "column": 0,
			      "line": 1,
			    },
			  },
			  "message": "  × Module not found: Can't resolve './answer' in '<TEST_ROOT>/fixtures/errors/resolve-fail-esm'   ╭──── 1 │ import { answer } from './answer'   · ─────────────────────────────────   ╰────  help: Did you mean './answer.js'?                The request './answer' failed to resolve only because it was resolved as fully specified,        probably because the origin is strict EcmaScript Module,        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.                The extension in the request is mandatory for it to be fully specified.        Add the extension to the request.",
			  "module": NormalModule {
			    "buildInfo": KnownBuildInfo {},
			    "buildMeta": Object {},
			    "context": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm",
			    "factoryMeta": Object {},
			    "layer": undefined,
			    "loaders": Array [],
			    "matchResource": undefined,
			    "rawRequest": "./resolve-fail-esm",
			    "request": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			    "resource": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			    "resourceResolveData": ReadonlyResourceData {
			      "fragment": "",
			      "path": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			      "query": "",
			      "resource": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			    },
			    "type": "javascript/esm",
			    "useSimpleSourceMap": false,
			    "useSourceMap": false,
			    "userRequest": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			  },
			  "name": "Error",
			  "stack": undefined,
			},
			]
		`);
		}
	},
	{
		description:
			"Testing map function on errors and warnings: test map of errors",
		options() {
			errors = [];
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test errors map", compilation => {
							compilation.errors.push(new Error(""));
							compilation.errors = compilation.errors.filter(
								item => item.message
							);

							errors = compilation.errors.map((item, index) => {
								item.index = index;
								return item;
							});
						});
					}
				]
			};
		},
		async check() {
			expect(errors).toMatchInlineSnapshot(`
			Array [
			  Object {
			  "index": 0,
			  "loc": Object {
			    "end": Object {
			      "column": 33,
			      "line": 1,
			    },
			    "start": Object {
			      "column": 0,
			      "line": 1,
			    },
			  },
			  "message": "  × Module not found: Can't resolve './answer' in '<TEST_ROOT>/fixtures/errors/resolve-fail-esm'   ╭──── 1 │ import { answer } from './answer'   · ─────────────────────────────────   ╰────  help: Did you mean './answer.js'?                The request './answer' failed to resolve only because it was resolved as fully specified,        probably because the origin is strict EcmaScript Module,        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.                The extension in the request is mandatory for it to be fully specified.        Add the extension to the request.",
			  "module": NormalModule {
			    "buildInfo": KnownBuildInfo {},
			    "buildMeta": Object {},
			    "context": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm",
			    "factoryMeta": Object {},
			    "layer": undefined,
			    "loaders": Array [],
			    "matchResource": undefined,
			    "rawRequest": "./resolve-fail-esm",
			    "request": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			    "resource": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			    "resourceResolveData": ReadonlyResourceData {
			      "fragment": "",
			      "path": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			      "query": "",
			      "resource": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			    },
			    "type": "javascript/esm",
			    "useSimpleSourceMap": false,
			    "useSourceMap": false,
			    "userRequest": "<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			  },
			  "name": "Error",
			  "stack": undefined,
			},
			  Object {
			  "index": 1,
			  "message": "  × ",
			  "name": "Error",
			  "stack": "Error:     at <TEST_ROOT>/errorCases/error-read.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			},
			]
		`);
		}
	}
];
