import { describeCases } from "./HotTestCases.template";

describeCases({
	name: "HotTestCasesNode",
	target: "node",
	casesPath: "hotCases",
	hot: true,
	incrementalRebuild: false
});
