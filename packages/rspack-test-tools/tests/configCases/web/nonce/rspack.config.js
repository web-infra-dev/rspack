/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		chunkFilename: "[name].js"
	},
	experiments: {
		css: true
	},
};
