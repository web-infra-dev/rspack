const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have build time executed",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/css/index",
			module: {
				rules: [
					{
						test: /\.css$/,
						use: [CssExtractRspackPlugin.loader, "css-loader"]
					}
				]
			},
			plugins: [
				new CssExtractRspackPlugin({
					filename: "[name].css"
				})
			],
			experiments: {
				css: false
			}
		};
	},
	async check(stats) {
		const statsOptions = {
			modules: true,
			runtimeModules: false,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson(statsOptions);
		const executedModules = statsJson.modules.filter(i => i.buildTimeExecuted);
		expect(executedModules.length).toBe(3);
		expect(JSON.parse(JSON.stringify(executedModules))).toMatchInlineSnapshot(`
		Array [
		  Object {
		    buildTimeExecuted: true,
		    built: true,
		    cacheable: true,
		    cached: false,
		    chunks: Array [],
		    codeGenerated: true,
		    errors: 0,
		    failed: false,
		    identifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		    moduleType: javascript/auto,
		    name: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		    nameForCondition: <TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		    optimizationBailout: Array [
		      Statement with side_effects in source code at ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css<LINE_COL>-100,
		    ],
		    optional: false,
		    orphan: true,
		    providedExports: null,
		    reasons: Array [
		      Object {
		        active: true,
		        moduleId: null,
		        resolvedModuleId: null,
		        type: loader import,
		        userRequest: <TEST_TOOLS_ROOT>/tests/fixtures/css/style.css.webpack[javascript/auto]!=!!!<ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		      },
		    ],
		    size: 754,
		    sizes: Object {
		      javascript: 754,
		    },
		    type: module,
		    usedExports: null,
		    warnings: 0,
		  },
		  Object {
		    buildTimeExecuted: true,
		    built: true,
		    cacheable: true,
		    cached: false,
		    chunks: Array [],
		    codeGenerated: true,
		    errors: 0,
		    failed: false,
		    identifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		    issuer: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		    issuerName: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		    issuerPath: Array [
		      Object {
		        identifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        name: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		      },
		    ],
		    moduleType: javascript/auto,
		    name: ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		    nameForCondition: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		    optimizationBailout: Array [
		      Statement with side_effects in source code at ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js<LINE_COL_RANGE>,
		    ],
		    optional: false,
		    orphan: true,
		    providedExports: null,
		    reasons: Array [
		      Object {
		        active: true,
		        loc: 3:0-239,
		        moduleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        moduleName: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModule: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModuleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        type: esm import,
		        userRequest: ../../../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		      },
		      Object {
		        active: true,
		        loc: 4:30-57,
		        moduleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        moduleName: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModule: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModuleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        type: esm import specifier,
		        userRequest: ../../../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		      },
		      Object {
		        active: true,
		        moduleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		        moduleName: ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		        resolvedModule: ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		        resolvedModuleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/api.js,
		        type: cjs self exports reference,
		        userRequest: self,
		      },
		    ],
		    size: 2303,
		    sizes: Object {
		      javascript: 2303,
		    },
		    type: module,
		    usedExports: null,
		    warnings: 0,
		  },
		  Object {
		    buildTimeExecuted: true,
		    built: true,
		    cacheable: true,
		    cached: false,
		    chunks: Array [],
		    codeGenerated: true,
		    errors: 0,
		    failed: false,
		    identifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		    issuer: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		    issuerName: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		    issuerPath: Array [
		      Object {
		        identifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        name: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		      },
		    ],
		    moduleType: javascript/auto,
		    name: ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		    nameForCondition: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		    optimizationBailout: Array [
		      Statement with side_effects in source code at ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js<LINE_COL_RANGE>,
		    ],
		    optional: false,
		    orphan: true,
		    providedExports: null,
		    reasons: Array [
		      Object {
		        active: true,
		        loc: 2:0-261,
		        moduleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        moduleName: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModule: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModuleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        type: esm import,
		        userRequest: ../../../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		      },
		      Object {
		        active: true,
		        loc: 4:58-98,
		        moduleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        moduleName: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModule: ./fixtures/css/style.css!=!../../../node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!./fixtures/css/style.css,
		        resolvedModuleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/cjs.js!<TEST_TOOLS_ROOT>/tests/fixtures/css/style.css,
		        type: esm import specifier,
		        userRequest: ../../../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		      },
		      Object {
		        active: true,
		        moduleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		        moduleName: ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		        resolvedModule: ../../../node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		        resolvedModuleIdentifier: <ROOT>/node_modules/<PNPM_INNER>/css-loader/dist/runtime/noSourceMaps.js,
		        type: cjs self exports reference,
		        userRequest: self,
		      },
		    ],
		    size: 64,
		    sizes: Object {
		      javascript: 64,
		    },
		    type: module,
		    usedExports: null,
		    warnings: 0,
		  },
		]
	`);
	}
};
