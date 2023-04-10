/** @type {import("../../../dist").Configuration} */
module.exports = {
	output: {
		filename: "[name].[contenthash].js"
	},
	optimization: {
		moduleIds: 'named',
		minimize: false,
		runtimeChunk: {
			name: 'runtime'
		}
	},
	
};
