/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		target: "web",
		mode: "development",
		output: {
			uniqueName: "my-app"
		},
		optimization: {
			chunkIds: 'named'
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		},

	},
	{
		target: "web",
		mode: "production",
		optimization: {
			chunkIds: 'named'
		},
		performance: false,
		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		},

	}
];
