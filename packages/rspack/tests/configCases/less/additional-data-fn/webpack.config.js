const lessLoader = require("@rspack/less-loader");

module.exports = {
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [
					{
						loader: lessLoader,
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
				type: "css"
			}
		]
	}
};
