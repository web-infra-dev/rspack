import { describeCases } from "./HotTestCases.template";

describeCases({
	name: "HotTestCasesWeb",
	target: "web",
	casesPath: "hotCases",
	hot: true
});
