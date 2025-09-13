/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/public/"
	},
	experiments: {
		outputModule: true
	},
	target: ["web", "es2020"]
};
