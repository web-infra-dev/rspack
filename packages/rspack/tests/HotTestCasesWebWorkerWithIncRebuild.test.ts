import { describeCases } from "./HotTestCases.template";

describeCases({
	name: "HotTestCasesWebWorkerWithIncrementalRebuild",
	target: "webworker",
	casesPath: "hotCases",
	hot: true,
	incrementalRebuild: true
});
