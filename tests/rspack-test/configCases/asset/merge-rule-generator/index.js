import url from "./data.json" with { type: "download" };

it("should apply generator options from a separate matching rule", () => {
	expect(url).toMatch(/^data\/data\.[a-f0-9]+\.json$/);
});
