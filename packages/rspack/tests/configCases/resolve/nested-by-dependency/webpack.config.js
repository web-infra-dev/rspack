module.exports = {
	mode: "development",
	resolve: {
		byDependency: {
			esm: {
				extensions: [".bar", "..."] // enable resolve .bar
			}
		}
	},
	module: {
		rules: [
			{
				test: /\.bar$/,
				resolve: {
					byDependency: {
						// dependencyType of import is esm
						esm: {
							extensions: [".mjs", "..."], // enable resolve .mjs in .bar
							fullySpecified: false // resolve .mjs without fullySpecified in .bar
						}
					}
				}
			}
		]
	}
};
