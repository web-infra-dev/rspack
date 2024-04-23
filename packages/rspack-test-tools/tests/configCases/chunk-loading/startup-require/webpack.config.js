module.exports = {
	entry: {
		main: "./index.js",
		other: "./other.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "require"
	},
	optimization: {
		splitChunks: {
			minSize: 0,
			cacheGroups: {
				lib1: {
					test: /lib-1/,
					name: "lib1",
					chunks: "all",
					priority: 3
				},
				lib2: {
					test: /lib-2/,
					name: "lib2",
					chunks: "all",
					priority: 2
				},
				lib3: {
					test: /lib-3/,
					name: "lib3",
					chunks: "all",
					priority: 1
				}
			}
		}
	},
	target: "node"
};
