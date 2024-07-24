/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	module: {
		rules: [
			{
				dependency: "url",
				type: "asset"
			}
		]
	}
};
