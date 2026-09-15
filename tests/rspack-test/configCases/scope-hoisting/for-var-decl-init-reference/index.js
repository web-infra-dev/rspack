import generate from "./generate";

it("should preserve references in for statement variable initializers", () => {
	expect(generate()).toBe(15);
});
