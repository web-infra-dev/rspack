/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /[ab]\.js$/,
				use: function (data) {
					return {
						loader: "./loader",
						// DIFF: need to use ident to identify the loader options
						ident: data.resource,
						options: {
							resource: data.resource.replace(/^.*[\\/]/g, ""),
							resourceQuery: data.resourceQuery,
							issuer: data.issuer.replace(/^.*[\\/]/g, "")
						}
					};
				}
			}
		]
	}
};
