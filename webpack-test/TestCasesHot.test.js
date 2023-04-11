const { describeCases } = require("./TestCases.template");
const webpack = require("@rspack/core").rspack;

describe("TestCases", () => {
	describeCases({
		name: "hot",
		plugins: [new webpack.HotModuleReplacementPlugin()]
	});
});
