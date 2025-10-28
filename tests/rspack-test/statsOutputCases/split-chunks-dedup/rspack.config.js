const webpack = require("@rspack/core");
const { ModuleFederationPlugin } = webpack.container;

const chunkIdChunkNameMap = new Map();
const usedSharedModuleNames = new Set();

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./"
	},
	mode: "production",
	optimization: {
		splitChunks: {
			cacheGroups: {
				defaultVendors: false,
				main: {
					name: "main",
					enforce: true,
					minChunks: 3
				}
			}
		}
	},
	output: {
		chunkFilename(pathData) {
			const { chunk } = pathData;
			if (chunk && "groupsIterable" in chunk) {
				for (const group of chunk.groupsIterable) {
					if (group.origins) {
						for (const origin of group.origins) {
							if (
								// CHANGE: origin.module.type === WEBPACK_MODULE_TYPE_PROVIDE
								origin.module.type === "provide-module" &&
								chunk.id
							) {
								if (chunkIdChunkNameMap.has(chunk.id)) {
									return `${chunkIdChunkNameMap.get(chunk.id)}.js`;
								}

								// @ts-expect-error
								const sharedModuleName = origin.module._name;
								const chunkName = `${sharedModuleName}-${chunk.id}--shared`;
								usedSharedModuleNames.add(sharedModuleName);
								chunkIdChunkNameMap.set(chunk.id, chunkName);

								return `${chunkName}.js`;
							}
						}
					}
				}
			}
			return "[id]--chunk.js";
		}
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
		}),
		new webpack.optimize.MergeDuplicateChunksPlugin({
			stage: 10
		})
	],
	stats: {
		assets: true
	}
};
