const { describeCases } = require("./TestCases.template");
const webpack = require("@rspack/core");

describe("TestCases", () => {
	describeCases({
		name: "hot",
		plugins: [new webpack.HotModuleReplacementPlugin()]
	});
});
