/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				resourceQuery: /^\?loader/,
				use: "./loader?query"
			}
		]
	}
};
