/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				issuerLayer: "dark",
				resolve: {
					conditionNames: ["dark", "..."]
				}
			}
		]
	}
};
