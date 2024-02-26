module.exports = {
	entry: {
		a: './a/index.js',
		b: './b/index.js',
		main: './main/index.js'
	},
	output: {
		filename: '[name].js',
		// FIXME: throws an error when chunkFilename = "[name].chunk.[fullhash].js"
		// Record in issue #5752
		chunkFilename: '[name].js',
	},
	optimization: {
		runtimeChunk: 'single'
	}
}
