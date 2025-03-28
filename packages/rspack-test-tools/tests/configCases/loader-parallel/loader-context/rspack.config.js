
/**
	* @type {import('@rspack/core').Configuration}
	*/
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js$/,
				use: [
					{
						loader: './loader.js',
						parallel: true,
						options: {}
					}
				]
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
}
