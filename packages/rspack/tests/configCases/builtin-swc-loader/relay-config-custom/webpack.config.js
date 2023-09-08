const { createSwcLoaderExperiments } = require("@rspack/core");

module.exports = {
	module: {
		rules: [
			{
				test: /\.jsx?$/,
				loader: "builtin:swc-loader",
				options: {
					rspackExperiments: createSwcLoaderExperiments().useRelay(__dirname, {
						language: "typescript",
						artifactDirectory: "./custom"
					})
				}
			}
		]
	}
};
