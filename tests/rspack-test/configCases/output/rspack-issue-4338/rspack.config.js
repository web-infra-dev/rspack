/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	optimization: {
		chunkIds: "named",
		// inlineExports will evaluate top-level const, import() in a/index.js and b/index.js will be a ImportDependency instead of ImportContextDependency, so the generated files check will fail
		inlineExports: false,
	},
	output: {
		chunkFilename: "[name].js"
	},
	node: {
		__dirname: false
	}
};
