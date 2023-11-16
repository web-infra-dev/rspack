import a from "./a";
import b from "./b";
import c from "./c";

it("should handle default exports when used as value", function () {
	expect(a).toBe("a");
});

it("should handle default exports when used as function", function () {
	expect(b()).toBe("b");
});

it("should handle default exports when used as class", function () {
	let ins = new c();
	expect(ins.value()).toBe("c");
});

it("should handle default exports when used as class method callee", function () {
	expect(new c().value()).toBe("c");
});
