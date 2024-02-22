/** @type {import("../../../../").Configuration} */
module.exports = [
	{
		entry: "./a",
		target: "web",
		output: {
			filename: "a.js",
			scriptType: "module",
			publicPath: "auto"
		},
		module: {
			rules: [
				{
					test: /\.png$/,
					type: "asset/resource"
				}
			]
		}
	},
	{
		entry: "./index"
	}
];
