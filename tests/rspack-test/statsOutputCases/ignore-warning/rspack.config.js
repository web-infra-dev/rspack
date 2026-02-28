/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: "errors-warnings",
	ignoreWarnings: [
		/Using \/ for division outside/,
		{
			message: /ESModulesLinkingWarning/
		},
		{
			module: /a.js/
		},
		warning => {
			return warning.module.identifier().includes("b.js");
		}
	],
	module: {
		parser: {
			javascript: {
				exportsPresence: 'auto',
			}
		},
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
