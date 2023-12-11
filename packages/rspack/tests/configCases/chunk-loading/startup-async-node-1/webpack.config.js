module.exports = {
	entry: {
		main: "./index.js",
		async: "./async.js",
		other: "./other.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "async-node",
		library: {
			name: "MyLib",
			type: "commonjs-module"
		}
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
				}
			}
		}
	},
	target: "node"
};
