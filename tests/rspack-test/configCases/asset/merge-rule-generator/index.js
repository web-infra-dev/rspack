import url from "./data.json" with { type: "download" };
import inlineUrl from "./data.json" with { type: "inline" };

it("should apply generator options from a separate matching rule", () => {
	expect(url).toMatch(/^data\/data\.[a-f0-9]+\.json$/);
	expect(inlineUrl).toMatch(/^data:application\/x-rspack-dom,/);
});
