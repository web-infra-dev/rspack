/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		experiments: {
			outputModule: true
		},
		devtool: false,
		target: "web"
	},
	{
		experiments: {
			outputModule: true
		},
		devtool: false,
		target: "node10"
	}
];
