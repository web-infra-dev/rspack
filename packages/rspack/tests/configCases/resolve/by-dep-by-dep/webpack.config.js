module.exports = {
	mode: "development",
	resolve: {
		byDependency: {
			esm: {
				extensions: [".bar", ".mjs", "..."] // enable .bar and .mjs extensions
			}
		}
	},
	module: {
		rules: [
			{
				test: /\.bar$/,
				resolve: {
					byDependency: {
						esm: {
							fullySpecified: false // foo.bar import bar.mjs without fullySpecified
						}
					}
				}
			}
		]
	}
};
