let errors = [];

/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description:
		"Testing map function on errors and warnings: test map of errors",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test errors map", compilation => {
						errors = compilation.errors.map((item, index) => ({
							index,
							...item
						}));
					});
				}
			]
		};
	},
	async check() {
		expect(errors).toMatchInlineSnapshot(`
		Array [
		  Object {
		    code: GenericFailure,
		    error: Object {
		  "code": "GenericFailure",
		  "message": "Can't resolve './answer' in '<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm'",
		  "name": "Error",
		  "stack": "Error: Can't resolve './answer' in '<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm'\\n    at Proxy.map (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at <TEST_TOOLS_ROOT>/tests/errorCases/error-map.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
		},
		    index: 0,
		    loc: 1:0-33,
		    module: NormalModule {
		      _readableIdentifier: ./index.js,
		      buildInfo: BuildInfo {},
		      buildMeta: Object {},
		      context: <TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm,
		      factoryMeta: Object {},
		      layer: undefined,
		      loaders: Array [],
		      matchResource: undefined,
		      rawRequest: ./resolve-fail-esm,
		      request: <TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js,
		      resource: <TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js,
		      resourceResolveData: Object {
		        descriptionFileData: Object {
		          type: module,
		        },
		        descriptionFilePath: <TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm,
		        fragment: ,
		        path: <TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js,
		        query: ,
		        resource: <TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js,
		      },
		      type: javascript/esm,
		      useSimpleSourceMap: false,
		      useSourceMap: false,
		      userRequest: <TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js,
		      Symbol(): javascript/esm|<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js,
		    },
		    moduleIdentifier: javascript/esm|<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js,
		    name: ModuleNotFoundError,
		  },
		]
	`);
	}
};
