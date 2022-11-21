module.exports = {
	entry: {
		main: ["./index.js"]
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				custom: {
					chunks: "all",
					name: "vendor",
					test: "foo"
				}
			}
		}
	}
};
