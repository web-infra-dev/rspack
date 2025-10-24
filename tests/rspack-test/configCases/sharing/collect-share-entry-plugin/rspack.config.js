
const { sharing } = require("@rspack/core");

const { CollectShareEntryPlugin, SharePlugin } = sharing;

const sharedOptions = [
	[
		"xreact",
		{
			import: "xreact",
			shareKey: "xreact",
			shareScope: "default",
			version: "1.0.0"
		}
	]
];

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new SharePlugin({
			shareScope: "default",
			shared: {
				xreact: {
					import: "xreact",
					shareKey: "xreact",
					shareScope: "default",
					version: "1.0.0"
				}
			},
		}),
		new CollectShareEntryPlugin({
			sharedOptions
		})
	]
};
