/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		name: "default",
		entry: "./index",
		target: "node",
		output: {
			filename: "default-[name].js",
			libraryTarget: "commonjs2"
		},
		optimization: {
			splitChunks: {
				minSize: 1,
				chunks: "all"
			}
		}
	},
	{
		name: "many-vendors",
		entry: "./index",
		target: "node",
		output: {
			filename: "many-vendors-[name].js",
			libraryTarget: "commonjs2"
		},
		optimization: {
			splitChunks: {
				minSize: 1,
				chunks: "all",
				maxInitialRequests: Infinity,
				cacheGroups: {
					default: false,
					defaultVendors: false,
					vendors: {
						test: m => {
							const match = m.nameForCondition().match(/([b-d]+)\.js$/);
							if (match) return true;
						}
					}
				}
			}
		}
	}
];
