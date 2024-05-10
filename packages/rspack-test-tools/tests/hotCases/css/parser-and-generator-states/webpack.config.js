/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module",
        parser: {
          namedExports: false,
        },
				generator: {
					exportsConvention: "camel-case",
					localIdentName: "[path][name][ext]__[local]",
					exportsOnly: false,
				},
			}
		]
	}
};
