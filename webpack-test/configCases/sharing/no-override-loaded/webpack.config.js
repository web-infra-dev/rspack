// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = require("../../../../").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		uniqueName: "b"
	},
	plugins: [
		new SharePlugin({
			shared: ["package"]
		})
	]
};
