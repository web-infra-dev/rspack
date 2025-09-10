/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		/**
		 * After merge, the generator will be:
		 * {
		 *   "asset/resource": {
		 *     emit: true,
		 *     outputPath: 'assets/',
		 *     filename: '[name][ext]',
		 *     publicPath: 'https://cdn/assets/'
		 *   }
		 * }
		 */
		generator: {
			asset: {
				emit: false,
				outputPath: "assets/"
			},
			"asset/resource": {
				emit: true,
				filename: "[name][ext]"
			}
		},
		rules: [
			{
				test: /\.txt$/,
				type: "asset/resource",
				generator: {
					publicPath: "https://cdn/assets/"
				}
			}
		]
	}
};
