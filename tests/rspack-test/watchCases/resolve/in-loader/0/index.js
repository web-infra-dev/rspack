import value from "./loader.mjs!./loader.mjs";

it("should resolve to the correct file", () => {
	expect(value).toBe(`${WATCH_STEP};`);
});
