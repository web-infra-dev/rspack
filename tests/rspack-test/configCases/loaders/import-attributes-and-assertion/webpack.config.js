
// Rspack don't support assert since it's deprecated

/** @type {import("../../../../").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				with: { type: "json" },
				loader: require.resolve("./loader-with.js")
			}
		]
	}
};
