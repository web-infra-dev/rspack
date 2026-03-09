var webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		optimization: {
			moduleIds: false
		},
		plugins: [
			new webpack.HashedModuleIdsPlugin({
				hashFunction: "md4",
				hashDigest: "hex",
				hashDigestLength: 8
			})
		]
	},
	{
		optimization: {
			moduleIds: false
		},
		plugins: [
			new webpack.HashedModuleIdsPlugin({
				hashFunction: "xxhash64",
				hashDigest: "base62",
				hashDigestLength: 6
			})
		]
	}
];
