module.exports = {
	optimization: {
		moduleIds: "named",
		chunkIds: "named",
		usedExports: false,
		concatenateModules: false,
		splitChunks: {
			cacheGroups: {
				default: false,
				defaultVendors: false,
				shared: { test: /[\\/]dep\.js$/, name: "shared", enforce: true }
			}
		}
	}
};
