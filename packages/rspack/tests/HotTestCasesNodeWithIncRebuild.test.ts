import { describeCases } from "./HotTestCases.template";

describeCases({
	name: "HotTestCasesNodeWithIncrementalRebuild",
	target: "node",
	casesPath: "hotCases",
	hot: true,
	incrementalRebuild: true
});
