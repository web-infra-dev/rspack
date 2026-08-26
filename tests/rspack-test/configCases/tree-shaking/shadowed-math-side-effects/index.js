import "./shadowed-math";

it("should preserve calls on a shadowed Math binding", () => {
	expect(globalThis.__SHADOWED_MATH_CALLED__).toBe(true);
});
