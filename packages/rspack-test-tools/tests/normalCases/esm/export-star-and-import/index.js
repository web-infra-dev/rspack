import a from "./a";
export * from "./a";

it("should work well when use export star and import at same time", function () {
	expect(a).toBe(1);
});
