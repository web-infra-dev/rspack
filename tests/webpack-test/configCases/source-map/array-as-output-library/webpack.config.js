/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "source-map",
	output: {
		library: ["Foo", "[name]"]
	}
};
