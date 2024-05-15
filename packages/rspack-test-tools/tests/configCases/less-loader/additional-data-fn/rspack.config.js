/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [
					{
						loader: "less-loader",
						options: {
							additionalData: (content, loaderContext) => {
								const { resourcePath, rootContext } = loaderContext;
								const relativePath = require("path").relative(
									rootContext,
									resourcePath
								);

								return `
										@background: coral;
										${content};
										.custom-class {
											color: red;
											relative-path: '${relativePath}';
										};
									`;
							}
						}
					}
				],
				type: "css",
				generator: {
					exportsOnly: false,
				}
			}
		]
	}
};
