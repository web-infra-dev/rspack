import { other, val2c, Test } from "./shared";

it("should have correct runtime id", () => {
	expect(other).toBe("other");
	expect(val2c).toBe(42);
	expect(Test).toBeTypeOf("function");
	expect(new Test()).toBeInstanceOf(Test);
	expect(__webpack_require__.j).toBe("b-runtime");
});
