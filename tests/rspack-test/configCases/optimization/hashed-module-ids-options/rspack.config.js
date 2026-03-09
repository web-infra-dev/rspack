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
	}
];
