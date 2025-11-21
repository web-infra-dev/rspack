const { DllReferencePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new DllReferencePlugin({
			name: "function(id) { return {default: 'ok'}; }",
			scope: "dll",
			content: {
				"./module": {
					id: 1,
					buildMeta: {
						exportsType: "namespace"
					},
					exports: ["default"]
				}
			}
		}),
	]
};
