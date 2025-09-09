const { DefinePlugin, ProvidePlugin } = require("@rspack/core");

test("Should provide builtin plugins with correct class name", () => {
	expect(new DefinePlugin({}).constructor.name).toEqual("DefinePlugin");
	expect(new ProvidePlugin({}).constructor.name).toEqual("ProvidePlugin");
});
