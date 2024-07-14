const { CssExtractRspackPlugin } = require("@rspack/core");
const path = require("path");
const RSPACK_ROOT = path.resolve(__dirname, "../../../..");

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
		const replacePath = (_, str) =>
			typeof str === "string"
				? str.split(RSPACK_ROOT).join("<RSPACK_ROOT>")
				: str;
		expect(JSON.parse(JSON.stringify(executedModules, replacePath)))
			.toMatchInlineSnapshot(`
		Array [
		  Object {
		    "buildTimeExecuted": true,
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": true,
		    "errors": 0,
		    "failed": false,
		    "identifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js",
		    "issuer": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		    "issuerName": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		    "issuerPath": Array [
		      Object {
		        "identifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		        "name": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		      },
		    ],
		    "moduleType": "javascript/auto",
		    "name": "../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js",
		    "nameForCondition": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js",
		    "optimizationBailout": Array [
		      "Statement with side_effects in source code at ../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js:7:0-85:2",
		    ],
		    "optional": false,
		    "orphan": true,
		    "providedExports": null,
		    "reasons": Array [
		      Object {
		        "moduleIdentifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import",
		        "userRequest": "../../../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js",
		      },
		      Object {
		        "moduleIdentifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import specifier",
		        "userRequest": "../../../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js",
		      },
		      Object {
		        "moduleIdentifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js",
		        "moduleName": "../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/api.js",
		        "type": "cjs self exports reference",
		        "userRequest": "self",
		      },
		    ],
		    "size": 2303,
		    "sizes": Object {
		      "javascript": 2303,
		    },
		    "type": "module",
		    "usedExports": null,
		    "warnings": 0,
		  },
		  Object {
		    "buildTimeExecuted": true,
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": true,
		    "errors": 0,
		    "failed": false,
		    "identifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		    "issuer": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		    "issuerName": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		    "issuerPath": Array [
		      Object {
		        "identifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		        "name": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		      },
		    ],
		    "moduleType": "javascript/auto",
		    "name": "../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		    "nameForCondition": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		    "optimizationBailout": Array [
		      "Statement with side_effects in source code at ../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js:3:0-5:2",
		    ],
		    "optional": false,
		    "orphan": true,
		    "providedExports": null,
		    "reasons": Array [
		      Object {
		        "moduleIdentifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import",
		        "userRequest": "../../../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		      },
		      Object {
		        "moduleIdentifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import specifier",
		        "userRequest": "../../../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		      },
		      Object {
		        "moduleIdentifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		        "moduleName": "../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		        "type": "cjs self exports reference",
		        "userRequest": "self",
		      },
		    ],
		    "size": 64,
		    "sizes": Object {
		      "javascript": 64,
		    },
		    "type": "module",
		    "usedExports": null,
		    "warnings": 0,
		  },
		  Object {
		    "buildTimeExecuted": true,
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": true,
		    "errors": 0,
		    "failed": false,
		    "identifier": "<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		    "issuerPath": Array [],
		    "moduleType": "javascript/auto",
		    "name": "./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		    "nameForCondition": "<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		    "optimizationBailout": Array [
		      "Statement with side_effects in source code at ./fixtures/css/style.css!=!../../../node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css:4:0-100",
		    ],
		    "optional": false,
		    "orphan": true,
		    "providedExports": null,
		    "reasons": Array [
		      Object {
		        "type": "loader import",
		        "userRequest": "<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css.webpack[javascript/auto]!=!!!<RSPACK_ROOT>/node_modules/.pnpm/css-loader@6.11.0_@rspack+core@packages+rspack_webpack@5.92.0_webpack-cli@5.1.4_webpack@5.92.0__/node_modules/css-loader/dist/cjs.js!<RSPACK_ROOT>/packages/rspack-test-tools/tests/fixtures/css/style.css",
		      },
		    ],
		    "size": 710,
		    "sizes": Object {
		      "javascript": 710,
		    },
		    "type": "module",
		    "usedExports": null,
		    "warnings": 0,
		  },
		]
	`);
	}
};
