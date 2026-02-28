const { rspack } = require("@rspack/core");
const path = require("path");

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		target: "web",
		mode: "development",
		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		},

	},
	{
		target: "web",
		mode: "production",
		output: {
			uniqueName: "my-app"
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
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
	}
];
