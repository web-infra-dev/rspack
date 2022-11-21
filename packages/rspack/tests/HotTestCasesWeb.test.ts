import { describeCases } from "./HotTestCases.template";

describe("HotTestCases", () => {
	describeCases({
		name: "web",
		target: "web",
		casesPath: 'hotCases'
	});
});
