/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		assetModuleFilename: "images/failure[ext]"
	},
	module: {
		rules: [
			{
				test: /\.(png|jpg)$/,
				type: "asset/resource",
				rules: [
					{
						resourceQuery: "?custom1",
						generator: {
							filename: "custom-images/success1[ext]"
						}
					},

					{
						resourceQuery: "?custom2",
						generator: {
							filename: ({ filename }) => {
								if (filename.endsWith(".png?custom2")) {
									return "custom-images/success2[ext]";
								}
								return "images/failure2[ext]";
							}
						}
					}
				]
			}
		]
	}
};
