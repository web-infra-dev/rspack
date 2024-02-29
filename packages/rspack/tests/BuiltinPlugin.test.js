const { DefinePlugin, ProvidePlugin } = require("..");

test("Should provide builtin plugins with correct class name", () => {
	expect(new DefinePlugin({}).constructor.name).toEqual("DefinePlugin");
	expect(new ProvidePlugin({}).constructor.name).toEqual("ProvidePlugin");
});
