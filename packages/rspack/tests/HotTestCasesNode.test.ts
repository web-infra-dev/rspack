import { describeCases } from "./HotTestCases.template";

describe("HotTestCases", () => {
	describeCases({
		name: "node",
		target: "node",
		casesPath: "hotCases",
		hot: true
	});
});
