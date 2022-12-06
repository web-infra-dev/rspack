module.exports = {
	mode: "development",
	entry: {
		main: {
			import: ["./index.js"]
		}
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
