import a from "./a";
import b from "./b";
import c from "./c";
import d from "./d";
import e from "./e";

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
	expect(new c().a()).toBe("a");
});

it("should handle default exports when used as class property", function () {
	expect(new c().val).toBe("c");
	expect(new c().b).toBe("b");
});

it("should handle default exports when used as class from object property", function () {
	expect(new d.c().val).toBe("c");
	expect(new d.c().value()).toBe("c");
});

it("should handle default exports when used as class from function", function () {
	expect(new (e())().val).toBe("c");
	expect(new (e())().value()).toBe("c");
});
