import { other, Test, val2c } from "./concatenated-shared";

it("should have the correct value", () => {
	expect(other).toBe("other");
	expect(val2c).toBe(42);
	expect(Test).toBeTypeOf("function");
	expect(new Test()).toBeInstanceOf(Test);
});
