import { high, low, pair, template } from "./constants";

it("preserves surrogate code units in exported strings", () => {
	expect(high.charCodeAt(0)).toBe(0xd800);
	expect(low.charCodeAt(0)).toBe(0xde00);
	expect(pair.codePointAt(0)).toBe(0x1f600);
	expect(template.charCodeAt(2)).toBe(0xd800);
});

it("distinguishes a lone surrogate from the replacement character when folding conditions", () => {
	if ("\uD800" === "\uFFFD") throw new Error("Incorrectly folded condition");
});
