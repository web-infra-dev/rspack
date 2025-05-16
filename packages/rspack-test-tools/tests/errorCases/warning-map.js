let warnings = [];

/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description:
		"Testing map function on errors and warnings: test map of warnings",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test warnings map", compilation => {
						warnings = compilation.warnings.map((item, index) => ({
							index,
							...item
						}));
					});
				}
			]
		};
	},
	async check() {
		expect(warnings).toMatchInlineSnapshot(`
		Array [
		  Object {
		    error: Object {
		  "message": "Unsupported feature: require.main.require() is not supported by Rspack.",
		  "name": "Error",
		  "stack": "Error: Unsupported feature: require.main.require() is not supported by Rspack.\\n    at Compilation.get warnings [as warnings] (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at Object.fn (<TEST_TOOLS_ROOT>/tests/errorCases/warning-map.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
		},
		    index: 0,
		    module: NormalModule {
		      _readableIdentifier: ./require.main.require.js,
		      buildInfo: BuildInfo {},
		      buildMeta: Object {},
		      context: <TEST_TOOLS_ROOT>/tests/fixtures/errors,
		      factoryMeta: Object {},
		      layer: undefined,
		      loaders: Array [],
		      matchResource: undefined,
		      rawRequest: ./require.main.require,
		      request: <TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js,
		      resource: <TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js,
		      resourceResolveData: Object {
		        descriptionFileData: Object {
		          author: ,
		          license: ISC,
		          name: rspack-core-tests,
		          version: 1.0.0,
		        },
		        descriptionFilePath: <TEST_TOOLS_ROOT>/tests,
		        fragment: ,
		        path: <TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js,
		        query: ,
		        resource: <TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js,
		      },
		      type: javascript/auto,
		      useSimpleSourceMap: false,
		      useSourceMap: false,
		      userRequest: <TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js,
		      Symbol(): <TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js,
		    },
		    moduleIdentifier: <TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js,
		    name: ModuleParseWarning,
		  },
		]
	`);
	}
};
