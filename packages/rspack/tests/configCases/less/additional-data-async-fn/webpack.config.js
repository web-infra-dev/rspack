const lessLoader = require("@rspack/plugin-less").default;

module.exports = {
	module: {
		rules: [
			{
				test: ".less$",
				uses: [
					{
						loader: lessLoader,
						options: {
							additionalData: async (content, loaderContext) => {
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
