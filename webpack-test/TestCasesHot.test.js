const { describeCases } = require("./TestCases.template");
const webpack = require("@rspack/core");

describe("TestCases", () => {
	describeCases({
		name: "hot",
		// TODO: recover this line after we have this js plugin
		// plugins: [new webpack.HotModuleReplacementPlugin()]
	});
});
