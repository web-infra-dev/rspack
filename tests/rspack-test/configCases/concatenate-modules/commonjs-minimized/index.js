import {
	anonymousFunction,
	anonymousArrow,
	AnonymousClass
} from "./anonymous-name";

it("should preserve anonymous function and class names after minimizing", () => {
	expect(anonymousFunction.name).toBe("");
	expect(anonymousArrow.name).toBe("");
	expect(AnonymousClass.name).toBe("");
});
