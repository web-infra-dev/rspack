let warnings = [];

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
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
		  "stack": undefined,
		},
		    index: 0,
		    module: NormalModule {
		      buildInfo: KnownBuildInfo {},
		      buildMeta: Object {},
		      context: <TEST_ROOT>/fixtures/errors,
		      factoryMeta: Object {},
		      layer: undefined,
		      loaders: Array [],
		      matchResource: undefined,
		      rawRequest: ./require.main.require,
		      request: <TEST_ROOT>/fixtures/errors/require.main.require.js,
		      resource: <TEST_ROOT>/fixtures/errors/require.main.require.js,
		      resourceResolveData: ReadonlyResourceData {
		        fragment: ,
		        path: <TEST_ROOT>/fixtures/errors/require.main.require.js,
		        query: ,
		        resource: <TEST_ROOT>/fixtures/errors/require.main.require.js,
		      },
		      type: javascript/auto,
		      useSimpleSourceMap: false,
		      useSourceMap: false,
		      userRequest: <TEST_ROOT>/fixtures/errors/require.main.require.js,
		    },
		    name: ModuleParseWarning,
		  },
		]
	`);
	}
};
