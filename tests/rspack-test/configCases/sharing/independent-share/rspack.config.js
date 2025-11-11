const { IndependentSharePlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "named",
		moduleIds: "named"
	},
	output: {
		chunkFilename: "[id].js"
	},
	plugins: [
		new IndependentSharePlugin({
			name: 'independent_share',
			shared: {
				'ui-lib' : {}
			}
		})
	]
};
