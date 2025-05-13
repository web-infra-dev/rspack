/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent"
		},
		incremental: true
	},
	ignoreWarnings: [/not friendly for incremental/]
};
