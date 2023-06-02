import { describeCases } from "./HotTestCases.template";

describeCases({
	name: "HotTestCasesWebWorker",
	target: "webworker",
	casesPath: "hotCases",
	hot: true,
	incrementalRebuild: false
});
