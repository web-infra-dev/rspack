import wrapped from "./wrapped";

it("should execute modules with interception enabled", () => {
	expect(true).toBe(true);
	expect(wrapped).toBe(42);
});
