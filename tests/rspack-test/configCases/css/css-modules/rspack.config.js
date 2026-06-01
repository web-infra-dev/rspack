"use strict";

const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {NonNullable<import("@rspack/core").Configuration["module"]>["rules"]} */
const baseRules = [
	{
		test: /\.module\.css$/i,
		type: "css/auto"
	},
	{
		test: /\.my-css$/i,
		type: "css/auto"
	},
	{
		test: /\.invalid$/i,
		type: "css/auto"
	}
];

/** @type {import("@rspack/core").Configuration} */
const base = {
	module: {
		rules: baseRules
	}
};

// target: ["web", "node"]
// output module

/** @type {(env: Env, options: TestOptions) => import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		...base,
		name: "web-development",
		target: "web",
		mode: "development",
		output: {
			uniqueName: "my-app"
		},
		node: {
			__dirname: false,
			__filename: false
		}
	},
	{
		...base,
		name: "web-production",
		target: "web",
		mode: "production",
		output: {
			uniqueName: "my-app"
		},
		node: {
			__dirname: false,
			__filename: false
		},
		plugins: [
			new rspack.ids.DeterministicModuleIdsPlugin({
				maxLength: 3,
				failOnConflict: true,
				fixedLength: true,
				test: m => m.type.startsWith("css")
			}),
			new rspack.experiments.ids.SyncModuleIdsPlugin({
				test: m => m.type.startsWith("css"),
				path: path.resolve(testPath, "module-ids.json"),
				mode: "create"
			})
		]
	},
	{
		...base,
		dependencies: ["web-development"],
		name: "node-development",
		target: "node",
		mode: "development",
		output: {
			uniqueName: "my-app"
		}
	},
	{
		...base,
		dependencies: ["web-production"],
		name: "node-production",
		target: "node",
		mode: "production",
		output: {
			uniqueName: "my-app"
		},
		plugins: [
			new rspack.ids.DeterministicModuleIdsPlugin({
				maxLength: 3,
				failOnConflict: true,
				fixedLength: true,
				test: m => m.type.startsWith("css")
			}),
			new rspack.experiments.ids.SyncModuleIdsPlugin({
				test: m => m.type.startsWith("css"),
				path: path.resolve(testPath, "module-ids.json"),
				mode: "read"
			})
		]
	}

	// TODO: Enable the webpack `css/global` and parser-options compilers once
	// Rspack can parse the full `style.module.css` fixture without panicking in
	// css-module-lexer.
	// // CSS modules `css/global`
	// {
	// 	entry: "./index-global.js",
	// 	name: "web-development-global",
	// 	target: "web",
	// 	mode: "development",
	// 	module: {
	// 		rules: [
	// 			{
	// 				test: /\.css$/i,
	// 				type: "css/global"
	// 			},
	// 			{
	// 				test: /\.my-css$/i,
	// 				type: "css/global"
	// 			},
	// 			{
	// 				test: /\.invalid$/i,
	// 				type: "css/global"
	// 			}
	// 		]
	// 	},
	// 	output: {
	// 		uniqueName: "my-app"
	// 	},
	// 	node: {
	// 		__dirname: false,
	// 		__filename: false
	// 	}
	// },
	// {
	// 	entry: "./index-global.js",
	// 	name: "web-production-global",
	// 	target: "web",
	// 	mode: "production",
	// 	module: {
	// 		rules: [
	// 			{
	// 				test: /\.css$/i,
	// 				type: "css/global"
	// 			},
	// 			{
	// 				test: /\.my-css$/i,
	// 				type: "css/global"
	// 			},
	// 			{
	// 				test: /\.invalid$/i,
	// 				type: "css/global"
	// 			}
	// 		]
	// 	},
	// 	output: {
	// 		uniqueName: "my-app"
	// 	},
	// 	node: {
	// 		__dirname: false,
	// 		__filename: false
	// 	}
	// },
	// // CSS modules options
	// {
	// 	...base,
	// 	entry: "./index-options.js",
	// 	name: "web-development",
	// 	target: "web",
	// 	mode: "development",
	// 	output: {
	// 		uniqueName: "my-app"
	// 	},
	// 	module: {
	// 		rules: [
	// 			...baseRules,
	// 			{
	// 				test: /style\.module\.css$/,
	// 				type: "css/auto",
	// 				parser: {
	// 					animation: false,
	// 					customIdents: false,
	// 					dashedIdents: false,
	// 					container: false,
	// 					function: false,
	// 					grid: false
	// 				}
	// 			}
	// 		]
	// 	},
	// 	node: {
	// 		__dirname: false,
	// 		__filename: false
	// 	}
	// }
];
