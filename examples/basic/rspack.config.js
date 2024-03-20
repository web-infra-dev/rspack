/**@type{improt("@rspack/cli").Configuration}*/
module.exports = {
	optimization: {
		minimize: false,
		moduleIds: "named",
		chunkIds: "named"
	},
	entry: {
		main: "./index.ts"
	}
};
