/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js/,
				resolve: {
					conditionNames: ["server"]
				}
			},
			{
				test: /reexports\.js/,
				resolve: {
					alias: {
						"server-lib": "lib2"
					}
				}
			}
		]
	},
	resolve: {
		modules: ["modules"]
	}
};
