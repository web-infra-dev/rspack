import { describeCases } from "./HotTestCases.template";

describeCases({
	name: "HotTestCasesWebWithIncrementalRebuild",
	target: "web",
	casesPath: "hotCases",
	hot: true,
	incrementalRebuild: true
});
