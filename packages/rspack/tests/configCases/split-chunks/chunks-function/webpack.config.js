/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index",
		lib1: {
			import: "./lib1",
			library: {
				type: "commonjs2",
				name: "lib1"
			}
		},
		lib2: {
			import: "./lib2",
			library: {
				type: "commonjs2",
			}
		}
	},
	output: {
		filename: "[name].js"
	},
	target: "async-node",
	optimization: {
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				splitLib2: {
					chunks(chunk) {
						return chunk.name !== "lib1";
					},
					test: /shared\.js/,
					name: "shared"
				}
			}
		}
	}
};
