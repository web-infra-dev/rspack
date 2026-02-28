/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		output: {
			module: true,
		},
		devtool: false,
		target: "web"
	},
	{
		output: {
			module: true,
		},
		devtool: false,
		target: "node10"
	}
];
