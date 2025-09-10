/** @type {import("@rspack/core").experiments} */
const experiments = require("@rspack/core").experiments;

const binding = require(process.env.RSPACK_BINDING);
binding.registerBindingBuilderTestingPlugin();

const BindingBuilderTestingPlugin = experiments.createNativePlugin(
	"BindingBuilderTestingPlugin",
	options => options
);

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new BindingBuilderTestingPlugin({
			foo: "bar"
		})
	]
};
