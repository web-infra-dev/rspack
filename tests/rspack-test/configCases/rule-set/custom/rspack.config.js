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
						// Include query + issuer to avoid cross-request option cache hits when
						// the same resource is imported with different queries/issuers.
						ident: `${data.resource}${data.resourceQuery || ""}-${data.issuer}`,
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
