/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				issuer: /dark/,
				resolve: {
					conditionNames: ["dark", "..."]
				}
			}
		]
	}
};
