/** @type {import("../../../dist").Configuration} */
module.exports = {
	entry: "./index",
	mode: "production",
	entry: "./index",
	output: {
		filename: "bundle.js"
	},
	stats: {
		builtAt: false,
		timings: false,
		source: true,
		version: false
	}
}
