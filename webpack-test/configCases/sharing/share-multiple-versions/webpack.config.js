// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = require("../../../../").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
	},
	plugins: [
		new SharePlugin({
			shared: ["shared"]
		})
	]
};
