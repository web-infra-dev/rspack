/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					transformImport: [
						{
							libraryName: "./lib",
							customName: "./lib/{{ }}"
						}
					]
				}
			}
		]
	}
};
