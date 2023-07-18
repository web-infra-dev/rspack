/** @type {import("../../../dist").Configuration} */
module.exports = {
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
