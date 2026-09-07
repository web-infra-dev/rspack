import * as values from "./shared";

it("should remove runtime a from the dynamic import chunk", () => {
	expect(values.live).toBe("live");
});
