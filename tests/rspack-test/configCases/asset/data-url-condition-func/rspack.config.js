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
				dataUrlCondition: (source, { filename }) => {
					return filename.endsWith("img.png");
				}
			}
		}
	}
};
