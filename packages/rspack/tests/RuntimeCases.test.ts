import { describeCases as describeCasesForJsonp } from "./HotTestCases.template";
import { describeCases as describeCasesForNode } from "./case.template";

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
