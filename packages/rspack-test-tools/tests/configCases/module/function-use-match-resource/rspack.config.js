/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /__label__/,
				use: (info) => {
					return [
						{
							loader: "./loader.js",
							options: info,
						}
					]
				},
			}
		]
	}
};
