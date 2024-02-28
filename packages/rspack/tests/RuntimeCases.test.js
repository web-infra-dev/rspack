const {
	describeCases: describeCasesForJsonp
} = require("./HotTestCases.template");
const { describeCases: describeCasesForNode } = require("./TestCases.template");

describe("RuntimeTestCases", () => {
	describeCasesForJsonp({
		name: "RuntimeTestCases jsonp",
		target: "web",
		casesPath: "runtimeCases",
		hot: false
	});

	describeCasesForNode({
		name: "RuntimeTestCases node",
		casePath: "runtimeCases"
	});
});
