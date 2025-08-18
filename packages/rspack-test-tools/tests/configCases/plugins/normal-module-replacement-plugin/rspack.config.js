const { NormalModuleReplacementPlugin } = require("@rspack/core");
const path = require("node:path");

module.exports = /** @type {import("@rspack/core").Configuration} */ ({
	plugins: [
		new NormalModuleReplacementPlugin(/request.v1(\.|$)/, r => {
			r.request = r.request.replace(/request\.v1(\.|$)/, "request.v2$1");
		}),
		new NormalModuleReplacementPlugin(
			/resource\.foo\.js$/,
			({ createData }) => {
				if (createData && createData.resource) {
					createData.resource = createData.resource.replace(
						/resource\.foo\.js$/,
						"resource.bar.js"
					);
				}
			}
		),
		new NormalModuleReplacementPlugin(
			/[/\\]query[/\\]query.v1(\.|$)/,
			path.resolve(__dirname, "./query/query.v2.js")
		)
	]
});
