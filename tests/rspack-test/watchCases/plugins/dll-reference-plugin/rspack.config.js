var webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new webpack.DllReferencePlugin({
			name: "function(id) { return {default: 'ok'}; }",
			scope: "dll",
			content: {
				"./module": {
					id: 1,
					buildMeta: {
						exportsType: "namespace",
						providedExports: ["default"]
					}
				}
			}
		}),
	],
	ignoreWarnings: [/is not friendly for incremental/]
};
