const { NormalModuleReplacementPlugin } = require("@rspack/core");

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
		)
	]
});
