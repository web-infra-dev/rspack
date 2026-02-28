const { rspack } = require("@rspack/core");
const { ModuleFederationPluginV1: ModuleFederationPlugin } = rspack.container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		moduleIds: "named",
		chunkIds: "named"
	},
	plugins: [
		new ModuleFederationPlugin({
			shared: {
				table: {
					requiredVersion: "=1.0.0"
				},
				cell: {
					requiredVersion: "=1.0.0"
				},
				row: {
					requiredVersion: "=1.0.0"
				},
				templater: {
					requiredVersion: "=1.0.0"
				}
			}
		})
	]
};
