import { _A, _B, _C, _D } from "./class";

it("should handle class static block correctly", () => {
	expect((new _A()).prop).toBe("a");
	expect((new _B()).prop).toBe("b");
	expect((new _C()).prop).toBe("c");
	expect((new _D()).prop).toBe("d");
});
