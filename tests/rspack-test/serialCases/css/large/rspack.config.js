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
		experiments: {
			css: true
		}
	},
	{
		target: "web",
		mode: "production",
		optimization: {
			chunkIds: 'named'
		},
		performance: false,
		experiments: {
			css: true
		}
	}
];
