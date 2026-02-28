"use strict";

const { rspack } = require("@rspack/core");

/** @typedef {import("@rspack/core").GeneratorOptionsByModuleTypeKnown} GeneratorOptionsByModuleTypeKnown */

const common = {
	optimization: {
		chunkIds: "named"
	},
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module",
				oneOf: [
					{
						resourceQuery: /\?as-is$/,
						generator: {
							exportsConvention: "as-is"
						}
					},
					{
						resourceQuery: /\?camel-case$/,
						generator: {
							exportsConvention: "camel-case"
						}
					},
					{
						resourceQuery: /\?camel-case-only$/,
						generator: {
							exportsConvention: "camel-case-only"
						}
					},
					{
						resourceQuery: /\?dashes$/,
						generator: {
							exportsConvention: "dashes"
						}
					},
					{
						resourceQuery: /\?dashes-only$/,
						generator: {
							exportsConvention: "dashes-only"
						}
					},
					// {
					// 	resourceQuery: /\?upper$/,
					// 	/** @type {GeneratorOptionsByModuleTypeKnown["css/module"]} */
					// 	generator: {
					// 		exportsConvention: (name) => name.toUpperCase()
					// 	}
					// }
				]
			},
		]

	},

};

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		...common,
		mode: "development",
		target: "web",
		plugins: [
			new rspack.DefinePlugin({
				"process.env.TARGET": JSON.stringify("web")
			})
		]
	},
	{
		...common,
		mode: "production",
		target: "web",
		plugins: [
			new rspack.DefinePlugin({
				"process.env.TARGET": JSON.stringify("web")
			})
		]
	},
	{
		...common,
		mode: "development",
		target: "node",
		plugins: [
			new rspack.DefinePlugin({
				"process.env.TARGET": JSON.stringify("node")
			})
		]
	},
	{
		...common,
		mode: "production",
		target: "node",
		plugins: [
			new rspack.DefinePlugin({
				"process.env.TARGET": JSON.stringify("node")
			})
		]
	}
];
