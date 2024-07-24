/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				parser: {
					system: true
				}
			}
		]
	}
};
