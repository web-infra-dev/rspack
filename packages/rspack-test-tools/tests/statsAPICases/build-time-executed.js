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
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson(statsOptions);
		expect(statsJson.modules.filter(i => i.buildTimeExecuted))
			.toMatchInlineSnapshot(`
		Array [
		  Object {
		    "assets": undefined,
		    "buildTimeExecuted": true,
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": true,
		    "depth": undefined,
		    "errors": 0,
		    "failed": false,
		    "id": undefined,
		    "identifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/runtime/api.js",
		    "issuer": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		    "issuerId": undefined,
		    "issuerName": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		    "issuerPath": Array [
		      Object {
		        "id": undefined,
		        "identifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		        "name": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		      },
		    ],
		    "moduleType": "javascript/auto",
		    "name": "../../../node_modules/css-loader/dist/runtime/api.js",
		    "nameForCondition": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/runtime/api.js",
		    "optimizationBailout": Array [
		      "Statement with side_effects in source code at ../../../node_modules/css-loader/dist/runtime/api.js:7:0-85:2",
		    ],
		    "optional": false,
		    "orphan": true,
		    "postOrderIndex": undefined,
		    "preOrderIndex": undefined,
		    "providedExports": null,
		    "reasons": Array [
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import",
		        "userRequest": "../../../../../node_modules/css-loader/dist/runtime/api.js",
		      },
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import specifier",
		        "userRequest": "../../../../../node_modules/css-loader/dist/runtime/api.js",
		      },
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/runtime/api.js",
		        "moduleName": "../../../node_modules/css-loader/dist/runtime/api.js",
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
		    "assets": undefined,
		    "buildTimeExecuted": true,
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": true,
		    "depth": undefined,
		    "errors": 0,
		    "failed": false,
		    "id": undefined,
		    "identifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		    "issuer": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		    "issuerId": undefined,
		    "issuerName": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		    "issuerPath": Array [
		      Object {
		        "id": undefined,
		        "identifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		        "name": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		      },
		    ],
		    "moduleType": "javascript/auto",
		    "name": "../../../node_modules/css-loader/dist/runtime/noSourceMaps.js",
		    "nameForCondition": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		    "optimizationBailout": Array [
		      "Statement with side_effects in source code at ../../../node_modules/css-loader/dist/runtime/noSourceMaps.js:3:0-5:2",
		    ],
		    "optional": false,
		    "orphan": true,
		    "postOrderIndex": undefined,
		    "preOrderIndex": undefined,
		    "providedExports": null,
		    "reasons": Array [
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import",
		        "userRequest": "../../../../../node_modules/css-loader/dist/runtime/noSourceMaps.js",
		      },
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		        "moduleName": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		        "type": "esm import specifier",
		        "userRequest": "../../../../../node_modules/css-loader/dist/runtime/noSourceMaps.js",
		      },
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/runtime/noSourceMaps.js",
		        "moduleName": "../../../node_modules/css-loader/dist/runtime/noSourceMaps.js",
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
		    "assets": undefined,
		    "buildTimeExecuted": true,
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": true,
		    "depth": undefined,
		    "errors": 0,
		    "failed": false,
		    "id": undefined,
		    "identifier": "<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		    "issuer": undefined,
		    "issuerId": undefined,
		    "issuerName": undefined,
		    "issuerPath": Array [],
		    "moduleType": "javascript/auto",
		    "name": "./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css",
		    "nameForCondition": "<PROJECT_ROOT>/tests/fixtures/css/style.css",
		    "optimizationBailout": Array [
		      "Statement with side_effects in source code at ./fixtures/css/style.css!=!../../../node_modules/css-loader/dist/cjs.js!./fixtures/css/style.css:4:0-100",
		    ],
		    "optional": false,
		    "orphan": true,
		    "postOrderIndex": undefined,
		    "preOrderIndex": undefined,
		    "providedExports": null,
		    "reasons": Array [
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": undefined,
		        "moduleName": undefined,
		        "type": "loader import",
		        "userRequest": "<PROJECT_ROOT>/tests/fixtures/css/style.css.webpack[javascript/auto]!=!!!<HOME_DIR>/rspack-dev/rspack/node_modules/css-loader/dist/cjs.js!<PROJECT_ROOT>/tests/fixtures/css/style.css",
		      },
		    ],
		    "size": 478,
		    "sizes": Object {
		      "javascript": 478,
		    },
		    "type": "module",
		    "usedExports": null,
		    "warnings": 0,
		  },
		]
	`);
	}
};
