/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset"
			}
		],
		parser: {
			asset: {
				dataUrlCondition: {
					maxSize: 100 * 1024
				}
			}
		}
	}
};
