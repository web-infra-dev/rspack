const value = require("./input").default;

it("should reevaluate code filters when source changes", () => {
	expect(value).toBe(Number(WATCH_STEP) % 2 === 0 ? "__VALUE__" : "transformed");
});
