import { colorize, supportsBasicColor } from "./lib";

it("should handle `in` operator on concatenated external default imports", () => {
	expect(colorize("ok")).toBeTruthy();
	expect(typeof supportsBasicColor()).toBe("boolean");
});
