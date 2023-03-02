/** @type {import("../../../../").Configuration} */
module.exports = {
	output: {
		filename: "[name].js",
		library: {
			name: "MyLibraryRuntimeChunk",
			type: "assign"
		}
	},
	target: "node",
	optimization: {
		runtimeChunk: true
	}
};
